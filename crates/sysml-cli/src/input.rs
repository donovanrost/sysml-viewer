use crate::args::Options;
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};
use sysml_kernel::{AnalysisRequest, SourceDocument};

// Host-side read bounds prevent oversized allocation before kernel validation.
const FILE_BYTES: u64 = 5_000_000;
const REQUEST_BYTES: u64 = 25_000_000;

fn bounded_text(reader: impl Read, limit: u64, label: &str) -> Result<String, String> {
    let mut text = String::new();
    reader
        .take(limit + 1)
        .read_to_string(&mut text)
        .map_err(|e| format!("{label}: {e}"))?;
    if text.len() as u64 > limit {
        return Err(format!("{label}: exceeds {limit} bytes"));
    }
    Ok(text)
}

fn is_model(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("sysml") || e.eq_ignore_ascii_case("kerml"))
}

pub fn load(options: &Options) -> Result<AnalysisRequest, String> {
    if options.request_json {
        let input = bounded_text(io::stdin().lock(), REQUEST_BYTES, "JSON request")?;
        return serde_json::from_str(&input).map_err(|e| format!("Invalid request: {e}"));
    }
    let mut documents = Vec::new();
    let mut seen_files = HashSet::new();
    let mut seen_dirs = HashSet::new();
    let mut total = 0;
    for input in &options.paths {
        if input == "-" {
            if !is_model(Path::new(&options.stdin_name)) {
                return Err("--stdin-name must end in .sysml or .kerml".into());
            }
            let text = bounded_text(io::stdin().lock(), FILE_BYTES, "Standard input")?;
            add_document(
                &mut documents,
                &mut total,
                SourceDocument {
                    id: "stdin://model".into(),
                    name: options.stdin_name.clone(),
                    text,
                },
            )?;
            continue;
        }
        let mut pending = vec![PathBuf::from(input)];
        while let Some(path) = pending.pop() {
            let metadata = fs::metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let canonical =
                fs::canonicalize(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            if metadata.is_dir() {
                if !seen_dirs.insert(canonical) {
                    continue;
                }
                let mut entries = fs::read_dir(&path)
                    .map_err(|e| format!("{}: {e}", path.display()))?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                entries.sort_by_key(|e| e.file_name());
                for entry in entries.into_iter().rev() {
                    let kind = entry
                        .file_type()
                        .map_err(|e| format!("{}: {e}", entry.path().display()))?;
                    if kind.is_dir() && !entry.file_name().to_string_lossy().starts_with('.')
                        || kind.is_file() && is_model(&entry.path())
                    {
                        pending.push(entry.path());
                    }
                }
            } else if metadata.is_file() {
                if !is_model(&path) {
                    return Err(format!(
                        "{}: expected a .sysml or .kerml file",
                        path.display()
                    ));
                }
                if !seen_files.insert(canonical) {
                    continue;
                }
                if documents.len() == 32 {
                    return Err(
                        "At most 32 documents are supported; choose a smaller project directory"
                            .into(),
                    );
                }
                if metadata.len() > FILE_BYTES {
                    return Err(format!("{}: exceeds 5 MB", path.display()));
                }
                let name = path
                    .to_str()
                    .ok_or("Model paths must be valid UTF-8")?
                    .to_owned();
                let file = File::open(&path).map_err(|e| format!("{name}: {e}"))?;
                let text = bounded_text(file, FILE_BYTES, &name)?;
                add_document(
                    &mut documents,
                    &mut total,
                    SourceDocument {
                        id: name.clone(),
                        name,
                        text,
                    },
                )?;
            } else {
                return Err(format!(
                    "{}: expected a regular file or directory",
                    path.display()
                ));
            }
        }
    }
    if documents.is_empty() {
        return Err("No .sysml or .kerml files found".into());
    }
    Ok(AnalysisRequest {
        revision: 1,
        documents,
    })
}

fn add_document(
    documents: &mut Vec<SourceDocument>,
    total: &mut usize,
    document: SourceDocument,
) -> Result<(), String> {
    *total += document.text.len();
    if documents.len() == 32 || *total > 20_000_000 {
        return Err("Workspace exceeds 32 documents or 20 MB".into());
    }
    documents.push(document);
    Ok(())
}

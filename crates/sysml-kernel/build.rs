//! Embed the pinned library as text. No network or runtime filesystem access.
use std::{
    fs,
    path::{Path, PathBuf},
};

fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("vendored library directory") {
        let path = entry.expect("library entry").path();
        if path.is_dir() {
            collect(&path, files);
        } else if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("sysml" | "kerml")
        ) {
            files.push(path);
        }
    }
}

fn main() {
    let base = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../vendor/sysml-library/sources");
    println!("cargo:rerun-if-changed={}", base.display());
    let mut files = Vec::new();
    collect(&base, &mut files);
    files.sort();
    let sources: Vec<_> = files
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "name": p.strip_prefix(&base).unwrap().to_string_lossy().replace('\\', "/"),
                "text": fs::read_to_string(&p).unwrap()
            })
        })
        .collect();
    let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("library.json");
    fs::write(output, serde_json::to_vec(&sources).unwrap()).unwrap();
}

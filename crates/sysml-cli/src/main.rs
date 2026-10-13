use std::{
    fs,
    io::{self, Read},
    process::ExitCode,
};
use sysml_kernel::{AnalysisRequest, Engine, Severity, SourceDocument, ANALYSIS_STACK_BYTES};

fn run() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--help") {
        eprintln!("Usage: sysml-inspect [--strict] FILE.sysml [FILE.kerml ...]\n       sysml-inspect --request-json < request.json\nOutputs a schema-versioned analysis snapshot as JSON. --strict exits 1 on error diagnostics. Invalid input exits 2.");
        return Ok(true);
    }
    if let Some(flag) = args
        .iter()
        .find(|a| a.starts_with("--") && !matches!(a.as_str(), "--strict" | "--request-json"))
    {
        return Err(format!("Unknown option: {flag}"));
    }
    let strict = args.iter().any(|a| a == "--strict");
    let paths: Vec<_> = args.iter().filter(|a| !a.starts_with("--")).collect();
    let request = if args.iter().any(|a| a == "--request-json") {
        if !paths.is_empty() {
            return Err("--request-json cannot be combined with file paths".into());
        }
        let mut input = String::new();
        io::stdin()
            .take(25_000_001)
            .read_to_string(&mut input)
            .map_err(|e| e.to_string())?;
        if input.len() > 25_000_000 {
            return Err("Request exceeds 25 MB".into());
        }
        serde_json::from_str(&input).map_err(|e| format!("Invalid request: {e}"))?
    } else {
        if paths.is_empty() || paths.len() > 32 {
            return Err("Supply between 1 and 32 model files".into());
        }
        let mut documents = Vec::new();
        for name in paths {
            if fs::metadata(name)
                .map_err(|e| format!("{name}: {e}"))?
                .len()
                > 5_000_000
            {
                return Err(format!("{name}: exceeds 5 MB"));
            }
            documents.push(SourceDocument {
                id: name.clone(),
                name: name.clone(),
                text: fs::read_to_string(name).map_err(|e| format!("{name}: {e}"))?,
            });
        }
        AnalysisRequest {
            revision: 1,
            documents,
        }
    };
    let snapshot = Engine::bundled()
        .analyze(request, || false)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(io::stdout().lock(), &snapshot).map_err(|e| e.to_string())?;
    Ok(!strict
        || !snapshot
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error))
}

fn main() -> ExitCode {
    let result = std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(run)
        .expect("analysis thread")
        .join();
    match result {
        Ok(Ok(true)) => ExitCode::SUCCESS,
        Ok(Ok(false)) => ExitCode::from(1),
        Ok(Err(error)) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
        Err(_) => {
            eprintln!("Language engine failed");
            ExitCode::from(2)
        }
    }
}

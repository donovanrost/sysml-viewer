//! Filesystem/stdio adapter around the same kernel used by the viewer.
//! Command parsing, input discovery and output contracts stay separate from
//! language semantics; no parser or validation rules are implemented here.
mod args;
mod input;
mod output;

use args::{Command, Invocation};
use std::process::ExitCode;
use sysml_kernel::{
    check_syntax_batch, Engine, ANALYSIS_STACK_BYTES, ENGINE_VERSION, LIBRARY_VERSION,
    SCHEMA_VERSION,
};

fn run(legacy: bool) -> Result<u8, String> {
    let args = std::env::args_os()
        .skip(1)
        .map(|s| {
            s.into_string()
                .map_err(|_| "Arguments must be valid UTF-8".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let options = match args::parse(args, legacy)? {
        Invocation::Help(command) => {
            output::text(&args::help(command, legacy))?;
            return Ok(0);
        }
        Invocation::Version => {
            output::text(&format!("sysml {}\nKernel: {ENGINE_VERSION}\nLibrary: {LIBRARY_VERSION}\nSnapshot schema: {SCHEMA_VERSION}\nDiagnostic report schema: 1\n", env!("CARGO_PKG_VERSION")))?;
            return Ok(0);
        }
        Invocation::Run(options) => options,
    };
    let request = input::load(&options)?;
    let revision = request.revision;
    if options.command == Command::Parse {
        let reports = check_syntax_batch(request.documents).map_err(|e| e.to_string())?;
        let mut documents = Vec::new();
        let mut diagnostics = Vec::new();
        for report in reports {
            documents.push(report.document);
            diagnostics.extend(report.diagnostics);
        }
        return output::report(&options, revision, &documents, &mut diagnostics);
    }
    let documents = request.documents.clone();
    let mut snapshot = Engine::bundled()
        .analyze(request, || false)
        .map_err(|e| e.to_string())?;
    if options.command == Command::Inspect {
        let failed = output::failed(&options, &snapshot.diagnostics);
        output::json(&snapshot)?;
        Ok(u8::from(failed))
    } else {
        output::report(&options, revision, &documents, &mut snapshot.diagnostics)
    }
}

pub fn entry(legacy: bool) -> ExitCode {
    let result = std::thread::Builder::new()
        .name("sysml-analysis".into())
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(move || run(legacy));
    let result = match result {
        Ok(thread) => thread
            .join()
            .unwrap_or_else(|_| Err("Language engine failed".into())),
        Err(error) => Err(format!("Could not start analysis: {error}")),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("sysml: {}", output::terminal(&error));
            ExitCode::from(2)
        }
    }
}

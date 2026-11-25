use crate::args::{Command, Format, Options};
use serde::Serialize;
use std::{
    collections::HashMap,
    io::{self, Write},
};
use sysml_kernel::{Diagnostic, Severity, SourceDocument, ENGINE_VERSION, LIBRARY_VERSION};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report<'a> {
    schema_version: u32,
    command: &'static str,
    revision: u32,
    engine_version: &'static str,
    library_version: Option<&'static str>,
    documents: Vec<Document<'a>>,
    diagnostics: &'a [Diagnostic],
    summary: Summary,
    passed: bool,
}

#[derive(Serialize)]
struct Document<'a> {
    id: &'a str,
    name: &'a str,
}
#[derive(Serialize)]
struct Summary {
    files: usize,
    errors: usize,
    warnings: usize,
}

pub fn failed(options: &Options, diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|d| match d.severity {
        Severity::Error => options.strict || options.warnings_as_errors,
        Severity::Warning => options.warnings_as_errors,
    })
}

pub fn json(value: &impl Serialize) -> Result<(), String> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, value).map_err(|e| e.to_string())?;
    writeln!(stdout).map_err(|e| e.to_string())
}

pub fn text(value: &str) -> Result<(), String> {
    io::stdout()
        .lock()
        .write_all(value.as_bytes())
        .map_err(|e| e.to_string())
}

// Paths and messages may contain user-authored terminal control characters.
pub fn terminal(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}

pub fn report(
    options: &Options,
    revision: u32,
    documents: &[SourceDocument],
    diagnostics: &mut [Diagnostic],
) -> Result<u8, String> {
    diagnostics.sort_by(|a, b| {
        (
            &a.source.document_id,
            a.source.start.byte_offset,
            &a.code,
            &a.message,
        )
            .cmp(&(
                &b.source.document_id,
                b.source.start.byte_offset,
                &b.code,
                &b.message,
            ))
    });
    let failed = failed(options, diagnostics);
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let summary = Summary {
        files: documents.len(),
        errors,
        warnings: diagnostics.len() - errors,
    };
    if options.format == Format::Json {
        json(&Report {
            schema_version: 1,
            command: options.command.name(),
            revision,
            engine_version: ENGINE_VERSION,
            library_version: (options.command == Command::Validate).then_some(LIBRARY_VERSION),
            documents: documents
                .iter()
                .map(|d| Document {
                    id: &d.id,
                    name: &d.name,
                })
                .collect(),
            diagnostics,
            summary,
            passed: !failed,
        })?;
    } else {
        let names: HashMap<_, _> = documents
            .iter()
            .map(|d| (d.id.as_str(), d.name.as_str()))
            .collect();
        let mut stderr = io::stderr().lock();
        for d in diagnostics {
            let name = names
                .get(d.source.document_id.as_str())
                .copied()
                .unwrap_or(&d.source.document_id);
            let severity = if d.severity == Severity::Error {
                "error"
            } else {
                "warning"
            };
            writeln!(
                stderr,
                "{}:{}:{}: {severity}[{}]: {}",
                terminal(name),
                d.source.start.line,
                d.source.start.column,
                terminal(&d.code),
                terminal(&d.message)
            )
            .map_err(|e| e.to_string())?;
        }
        let operation = if options.command == Command::Parse {
            "Syntax checked"
        } else {
            "Validated"
        };
        text(&format!(
            "{operation} {} file(s): {} error(s), {} warning(s).\n",
            summary.files, summary.errors, summary.warnings
        ))?;
    }
    Ok(u8::from(failed))
}

use crate::{
    engine::{diagnostic, validate_documents},
    AnalysisError, Diagnostic, SourceDocument, SourceIndex, Stage,
};
use sysmlv2_syntax::parser;

/// Syntax/structure findings for one unchanged source. No libraries or name
/// resolution are involved. A recovered tree is internal to this operation;
/// this API does not expose an editable AST or promise lossless tree operations.
#[derive(Debug)]
pub struct SyntaxReport {
    pub document: SourceDocument,
    pub diagnostics: Vec<Diagnostic>,
}

/// Parse one supplied document and check body structure. Invalid language input
/// produces diagnostics; only input-policy violations return `AnalysisError`.
/// Hosts must use `ANALYSIS_STACK_BYTES`, just as for semantic analysis.
pub fn check_syntax(document: SourceDocument) -> Result<SyntaxReport, AnalysisError> {
    validate_documents(std::slice::from_ref(&document))?;
    Ok(parse_document(document))
}

/// Check independent sources under the same workspace input policy as semantic
/// analysis. This does not resolve references between the documents.
pub fn check_syntax_batch(
    documents: Vec<SourceDocument>,
) -> Result<Vec<SyntaxReport>, AnalysisError> {
    validate_documents(&documents)?;
    Ok(documents.into_iter().map(parse_document).collect())
}

fn parse_document(document: SourceDocument) -> SyntaxReport {
    let parsed = if document.name.to_ascii_lowercase().ends_with(".kerml") {
        parser::parse_kerml_source(&document.text)
    } else {
        parser::parse_source(&document.text)
    };
    let index = SourceIndex::new(&document.id, &document.text);
    let mut diagnostics: Vec<_> = parsed
        .diagnostics
        .iter()
        .map(|d| {
            diagnostic(
                d,
                Stage::Syntax,
                "syntax",
                index.range(d.span.start, d.span.end),
            )
        })
        .collect();
    diagnostics.extend(
        sysmlv2_syntax::check::validate(&parsed.unit)
            .iter()
            .map(|d| {
                diagnostic(
                    d,
                    Stage::Structure,
                    "structure",
                    index.range(d.span.start, d.span.end),
                )
            }),
    );
    SyntaxReport {
        document,
        diagnostics,
    }
}

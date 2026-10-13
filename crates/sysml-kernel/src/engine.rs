use std::collections::{HashMap, HashSet};

use sysmlv2_model::{
    check,
    json::{self, ElementRef, ResolvedModel},
    model::Model,
};
use sysmlv2_syntax::{diag, span::Span, token::TokenKind};

use crate::*;

#[derive(serde::Deserialize)]
struct LibrarySource {
    name: String,
    text: String,
}

/// Input-owned analysis service. Hosts own scheduling, timeouts and cancellation.
/// Current analysis rebuilds the workspace. No incremental-performance promise is
/// made by this API; snapshots remain usable after later requests complete.
pub struct Engine {
    library: Vec<LibrarySource>,
}

impl Engine {
    pub fn bundled() -> Self {
        Self {
            library: serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/library.json")))
                .expect("build-generated library"),
        }
    }

    /// Cancellation is checked between passes. It cannot interrupt an upstream
    /// pass; interactive hosts must use a disposable worker for hard cancellation.
    pub fn analyze(
        &self,
        request: AnalysisRequest,
        cancelled: impl Fn() -> bool,
    ) -> Result<Snapshot, AnalysisError> {
        let checkpoint = || {
            if cancelled() {
                Err(AnalysisError::Cancelled)
            } else {
                Ok(())
            }
        };
        checkpoint()?;
        validate_input(&request)?;
        let mut model = Model::new();
        let mut sources = Vec::new();
        for lib in &self.library {
            model.add_library_source(&lib.name, &lib.text);
            sources.push(SourceDocument {
                id: format!("library://{}", lib.name),
                name: lib.name.clone(),
                text: lib.text.clone(),
            });
        }
        let user_start = sources.len();
        for doc in &request.documents {
            checkpoint()?;
            // Keep model identity distinct from display name (two picked files may
            // share a basename). Dialect is still selected by the actual filename.
            let dialect = if doc.name.to_ascii_lowercase().ends_with(".kerml") {
                "kerml"
            } else {
                "sysml"
            };
            model.add_source(format!("{}.{}", doc.id, dialect), &doc.text);
            sources.push(doc.clone());
        }
        let indexes: Vec<_> = sources
            .iter()
            .map(|d| SourceIndex::new(&d.id, &d.text))
            .collect();
        let source = |unit: usize, span: Span| indexes[unit].range(span.start, span.end);
        // Upstream reference spans cover the qualified name, omitting a written
        // conjugation marker. Preserve it using lexer tokens, including trivia
        // between the marker and name, without changing semantic resolution.
        let conjugations: Vec<HashMap<u32, u32>> = sources[user_start..]
            .iter()
            .map(|doc| {
                let tokens: Vec<_> = sysmlv2_syntax::lexer::tokenize(&doc.text)
                    .0
                    .into_iter()
                    .filter(|t| !t.kind.is_trivia())
                    .collect();
                tokens
                    .windows(2)
                    .filter(|pair| pair[0].kind == TokenKind::Tilde)
                    .map(|pair| (pair[1].span.start, pair[0].span.start))
                    .collect()
            })
            .collect();
        let reference_span = |unit: usize, mut span: Span, kind: RelationshipKind| {
            if kind == RelationshipKind::Typing {
                if let Some(&start) = conjugations[unit - user_start].get(&span.start) {
                    span.start = start;
                }
            }
            span
        };
        let mut diagnostics = Vec::new();
        for (unit, parsed) in model.units().iter().enumerate().skip(user_start) {
            diagnostics.extend(
                parsed
                    .diagnostics
                    .iter()
                    .map(|d| diagnostic(d, Stage::Syntax, "syntax", source(unit, d.span))),
            );
            diagnostics.extend(
                sysmlv2_syntax::check::validate(&parsed.unit)
                    .iter()
                    .map(|d| diagnostic(d, Stage::Structure, "structure", source(unit, d.span))),
            );
        }
        checkpoint()?;
        let mut graph = ResolvedModel::build(&model);
        checkpoint()?;
        // The upstream public report is the authority on ambiguous vs absent
        // references. It currently builds a second graph; keep this cost visible
        // in benchmarks rather than infer classification from diagnostic prose.
        let report = json::model_resolution_report(&model);
        let ambiguous: HashSet<_> = report
            .ambiguous
            .iter()
            .map(|(u, q)| (*u, q.span.start, q.span.end))
            .collect();
        for (unit, d) in check::validate_model_with(&mut graph, &model) {
            if unit < user_start {
                continue;
            }
            let code = if ambiguous.contains(&(unit, d.span.start, d.span.end)) {
                "ambiguous-reference"
            } else {
                "resolution"
            };
            diagnostics.push(diagnostic(
                &d,
                Stage::Resolution,
                code,
                source(unit, d.span),
            ));
        }
        checkpoint()?;
        for (unit, d) in check::validate_semantics_with(&mut graph, &model) {
            if unit >= user_start {
                diagnostics.push(diagnostic(
                    &d,
                    Stage::Semantic,
                    "semantic",
                    source(unit, d.span),
                ));
            }
        }

        // Renderable declarations are namespaces/types/features with written
        // extents. Membership and relationship nodes remain engine internals.
        let declarations: Vec<_> = graph
            .user_elements()
            .filter(|e| {
                graph.member_extent(*e).is_some()
                    && check::metaclass_conforms(graph.element_type(*e), "Namespace")
            })
            .collect();
        if declarations.len() > 20_000 {
            return Err(AnalysisError::InvalidInput(
                "Workspace exceeds 20,000 declarations".into(),
            ));
        }
        let declaration_set: HashSet<_> = declarations.iter().copied().collect();
        let mut registry = Registry::new(request.revision);
        for e in graph.elements() {
            registry.id(e);
        }
        for &e in &declarations {
            registry.include(e);
        }
        let mut references = Vec::new();
        for site in graph.reference_sites().to_vec() {
            if site.unit < user_start {
                continue;
            }
            let owner = declaration_owner(&mut graph, site.owner, &declaration_set);
            registry.include(site.target);
            let kind = reference_kind(&site.kind);
            let span = reference_span(site.unit, site.span, kind);
            references.push(Reference {
                owner_id: owner.map(|e| registry.id(e)),
                kind,
                spelling: span.slice(&sources[site.unit].text).into(),
                resolution: Resolution::Resolved {
                    target_id: registry.id(site.target),
                },
                source: source(site.unit, span),
            });
        }
        let unresolved = graph.unresolved_references();
        for site in &unresolved {
            if site.unit < user_start {
                continue;
            }
            let owner = declaration_owner(&mut graph, site.owner, &declaration_set);
            let kind = relationship_kind(graph.element_type(site.owner));
            let span = reference_span(site.unit, site.span, kind);
            references.push(Reference {
                owner_id: owner.map(|e| registry.id(e)),
                kind,
                spelling: span.slice(&sources[site.unit].text).into(),
                resolution: if ambiguous.contains(&(site.unit, site.span.start, site.span.end)) {
                    Resolution::Ambiguous
                } else {
                    Resolution::Unresolved
                },
                source: source(site.unit, span),
            });
        }

        let mut member_queries = HashMap::new();
        let mut connections = Vec::new();
        for &e in &declarations {
            checkpoint()?;
            let owned: Vec<_> = graph
                .owned_members(e)
                .into_iter()
                .filter(|m| declaration_set.contains(m))
                .collect();
            let mut effective = owned.clone();
            for m in graph.effective_features(e, true) {
                if !effective.contains(&m) {
                    effective.push(m);
                }
                registry.include(m);
            }
            let complete = !graph.inheritance_incomplete(e, true);
            if !complete {
                let (unit, span) = graph.member_extent(e).unwrap();
                diagnostics.push(Diagnostic { code: "incomplete-inheritance".into(), stage: Stage::Capability, severity: Severity::Warning, message: "The engine could not completely determine this element's inherited members.".into(), source: source(unit, span) });
            }
            member_queries.insert(e, (owned, effective, complete));
            if check::metaclass_conforms(graph.element_type(e), "Connector") {
                let (unit, extent) = graph.member_extent(e).unwrap();
                let failed = unresolved.iter().any(|r| {
                    r.unit == unit && extent.start <= r.span.start && r.span.end <= extent.end
                });
                let ends = graph.connector_end_targets(e).into_iter().map(|end| {
                    let spelling = end.spelling.clone().unwrap_or_else(|| end.chain.iter().map(|&link| graph.element_effective_name(link).unwrap_or_default()).collect::<Vec<_>>().join("."));
                    for &link in &end.chain { registry.include(link); }
                    // Some upstream chains preserve a resolved prefix after a
                    // failed last step. Never label that prefix a resolved end.
                    let resolution = if failed {
                        Resolution::Unsupported { reason: "Connection contains an unresolved or ambiguous reference; endpoint completeness is not established.".into() }
                    } else if end.spelling.is_some() { Resolution::Unresolved }
                    else if let Some(&target) = end.chain.last() { Resolution::Resolved { target_id: registry.id(target) } }
                    else { Resolution::Unsupported { reason: "Implicit endpoint is not projected by this API.".into() } };
                    Endpoint { chain: end.chain.into_iter().map(|link| registry.id(link)).collect(), spelling, resolution }
                }).collect();
                connections.push(Connection {
                    element_id: registry.id(e),
                    ends,
                });
            }
        }

        // Include the owner chain of referenced/library declarations for source
        // navigation. Library records are inspection stubs, not expanded queries.
        let mut cursor = 0;
        while cursor < registry.included.len() {
            let e = registry.included[cursor];
            if let Some(owner) = graph.owner(e) {
                if graph.member_extent(owner).is_some() {
                    registry.include(owner);
                }
            }
            cursor += 1;
        }
        let mut elements = Vec::new();
        let mut used_units: HashSet<usize> = (user_start..sources.len()).collect();
        for e in registry.included.clone() {
            let parent = graph.owner(e).filter(|p| registry.selected.contains(p));
            let location = graph
                .member_extent(e)
                .or_else(|| graph.declaration_site(e))
                .map(|(u, s)| {
                    used_units.insert(u);
                    source(u, s)
                });
            let (owned, effective, complete) = member_queries.remove(&e).unwrap_or_default();
            let value_expression = graph.member_extent(e).and_then(|(u, _)| {
                graph
                    .value_expr(e)
                    .map(|(_, expr)| expr.span.slice(&sources[u].text).to_string())
            });
            let declared_multiplicity = graph.declared_multiplicity(e).map(|(lo, hi)| {
                let upper = if hi.is_infinite() {
                    "*".to_string()
                } else {
                    hi.to_string()
                };
                if lo == hi {
                    format!("[{upper}]")
                } else {
                    format!("[{lo}..{upper}]")
                }
            });
            elements.push(Element {
                id: registry.id(e),
                metaclass: graph.element_type(e).into(),
                name: graph
                    .element_effective_name(e)
                    .unwrap_or_else(|| format!("(anonymous {})", graph.element_type(e))),
                qualified_name: graph.element_qualified_name(e),
                parent_id: parent.map(|p| registry.id(p)),
                library: graph.is_library_element(e),
                source: location,
                documentation: graph
                    .element_docs(e)
                    .into_iter()
                    .map(|(_, body)| body)
                    .collect::<Vec<_>>()
                    .join("\n"),
                value_expression,
                declared_multiplicity,
                members_available: declaration_set.contains(&e),
                owned_members: owned.into_iter().map(|m| registry.id(m)).collect(),
                effective_members: effective.into_iter().map(|m| registry.id(m)).collect(),
                inheritance_complete: complete,
            });
        }
        references.sort_by(|a, b| {
            (
                &a.source.document_id,
                a.source.start.byte_offset,
                &a.spelling,
            )
                .cmp(&(
                    &b.source.document_id,
                    b.source.start.byte_offset,
                    &b.spelling,
                ))
        });
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
        diagnostics.dedup_by(|a, b| a.source == b.source && a.message == b.message);
        checkpoint()?;
        Ok(Snapshot {
            schema_version: SCHEMA_VERSION,
            revision: request.revision,
            engine_version: ENGINE_VERSION.into(),
            library_version: LIBRARY_VERSION.into(),
            documents: sources
                .into_iter()
                .enumerate()
                .filter_map(|(u, d)| used_units.contains(&u).then_some(d))
                .collect(),
            elements,
            references,
            connections,
            diagnostics,
        })
    }
}

fn validate_input(request: &AnalysisRequest) -> Result<(), AnalysisError> {
    let fail = |s: &str| AnalysisError::InvalidInput(s.into());
    if request.documents.len() > 32 {
        return Err(fail("At most 32 documents are supported"));
    }
    let mut ids = HashSet::new();
    let mut total = 0;
    let mut tokens = 0;
    for doc in &request.documents {
        if doc.id.is_empty() || doc.id.starts_with("library://") || !ids.insert(&doc.id) {
            return Err(fail(
                "Document IDs must be nonempty, unique and outside library://",
            ));
        }
        if ![".sysml", ".kerml"]
            .iter()
            .any(|ext| doc.name.to_ascii_lowercase().ends_with(ext))
        {
            return Err(fail("Expected a .sysml or .kerml document"));
        }
        total += doc.text.len();
        if doc.text.len() > 5_000_000 || total > 20_000_000 {
            return Err(fail("Sources exceed 5 MB per file or 20 MB per workspace"));
        }
        tokens += sysmlv2_syntax::lexer::tokenize(&doc.text).0.len();
        if tokens > 500_000 {
            return Err(fail("Workspace exceeds 500,000 syntax tokens"));
        }
    }
    Ok(())
}

fn diagnostic(
    d: &diag::Diagnostic,
    stage: Stage,
    fallback_code: &str,
    source: SourceRange,
) -> Diagnostic {
    Diagnostic {
        code: d.code.unwrap_or(fallback_code).into(),
        severity: match d.severity {
            diag::Severity::Error => Severity::Error,
            diag::Severity::Warning => Severity::Warning,
        },
        stage,
        message: d.message.clone(),
        source,
    }
}

fn declaration_owner(
    graph: &mut ResolvedModel,
    mut e: ElementRef,
    declarations: &HashSet<ElementRef>,
) -> Option<ElementRef> {
    let mut seen = HashSet::new();
    while seen.insert(e) {
        if declarations.contains(&e) {
            return Some(e);
        }
        e = graph.owner(e)?;
    }
    None
}

fn relationship_kind(metaclass: &str) -> RelationshipKind {
    match metaclass {
        "FeatureTyping" | "ConjugatedPortTyping" => RelationshipKind::Typing,
        "Subclassification" | "Specialization" => RelationshipKind::Specialization,
        "Subsetting" => RelationshipKind::Subsetting,
        "Redefinition" => RelationshipKind::Redefinition,
        "MembershipImport" | "NamespaceImport" => RelationshipKind::Import,
        "ReferenceSubsetting" | "FeatureChaining" => RelationshipKind::Reference,
        _ => RelationshipKind::Other,
    }
}

fn reference_kind(property: &str) -> RelationshipKind {
    match property {
        "type" => RelationshipKind::Typing,
        "general" | "superclassifier" => RelationshipKind::Specialization,
        "subsettedFeature" => RelationshipKind::Subsetting,
        "redefinedFeature" => RelationshipKind::Redefinition,
        "importedMembership" | "importedNamespace" => RelationshipKind::Import,
        "referencedFeature" | "chainingFeature" => RelationshipKind::Reference,
        _ => RelationshipKind::Other,
    }
}

struct Registry {
    revision: u32,
    ids: HashMap<ElementRef, String>,
    included: Vec<ElementRef>,
    selected: HashSet<ElementRef>,
}
impl Registry {
    fn new(revision: u32) -> Self {
        Self {
            revision,
            ids: HashMap::new(),
            included: Vec::new(),
            selected: HashSet::new(),
        }
    }
    fn id(&mut self, e: ElementRef) -> String {
        let next = self.ids.len();
        self.ids
            .entry(e)
            .or_insert_with(|| format!("s{}:e{next}", self.revision))
            .clone()
    }
    fn include(&mut self, e: ElementRef) {
        self.id(e);
        if self.selected.insert(e) {
            self.included.push(e);
        }
    }
}

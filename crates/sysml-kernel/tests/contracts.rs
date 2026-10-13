use std::sync::OnceLock;
use sysml_kernel::*;

fn document(id: &str, text: &str) -> SourceDocument {
    SourceDocument {
        id: id.into(),
        name: "Common.sysml".into(),
        text: text.into(),
    }
}

fn snapshot() -> &'static Snapshot {
    static VALUE: OnceLock<Snapshot> = OnceLock::new();
    VALUE.get_or_init(|| std::thread::Builder::new().stack_size(ANALYSIS_STACK_BYTES).spawn(|| {
        Engine::bundled().analyze(AnalysisRequest { revision: 41, documents: vec![
            document("definitions", "// 🚢\r\npackage Definitions { part def Base { attribute mass : ScalarValues::Real; } part def Empty; }"),
            document("usages", "package Usages { part x : Definitions::Base; part y : Definitions::Base; connection wire connect x to y; }"),
        ] }, || false).unwrap()
    }).unwrap().join().unwrap())
}

#[test]
fn member_queries_distinguish_empty_unknown_and_unavailable() {
    let query = snapshot().queries().unwrap();
    let empty = query.find("Definitions::Empty")[0];
    assert!(query.owned_members(&empty.id).unwrap().is_empty());
    assert!(matches!(
        query.owned_members("other-session"),
        Err(QueryError::UnknownElement(_))
    ));
    let real = query.find("ScalarValues::Real")[0];
    assert!(matches!(
        query.effective_members(&real.id),
        Err(QueryError::MembersUnavailable(_))
    ));
    assert!(matches!(
        query.references_from(&real.id),
        Err(QueryError::ReferencesUnavailable(_))
    ));
    let library_document = &real.source.as_ref().unwrap().document_id;
    assert!(matches!(
        query.diagnostics(library_document),
        Err(QueryError::DiagnosticsUnavailable(_))
    ));
}

#[test]
fn effective_query_keeps_library_members_and_ownership_separate() {
    let query = snapshot().queries().unwrap();
    let x = query.find("Usages::x")[0];
    assert!(query.owned_members(&x.id).unwrap().is_empty());
    let effective = query.effective_members(&x.id).unwrap();
    assert!(effective.inheritance_complete);
    assert!(effective
        .members
        .iter()
        .any(|e| e.qualified_name.as_deref() == Some("Definitions::Base::mass")));
    assert!(effective.members.iter().any(|e| e.library));
}

#[test]
fn canonical_lookup_does_not_claim_scoped_resolution_or_choose_first_match() {
    let mut value = snapshot().clone();
    let mut second = value.find("Definitions::Base")[0].clone();
    second.id = "second-canonical-name".into();
    value.elements.push(second);
    let query = value.queries().unwrap();
    assert!(query.find("mass").is_empty());
    assert_eq!(query.find("Definitions::Base").len(), 2);
}

#[test]
fn source_queries_keep_equal_filenames_distinct_and_use_half_open_ranges() {
    let query = snapshot().queries().unwrap();
    assert_ne!(
        query.document("definitions").unwrap().text,
        query.document("usages").unwrap().text
    );
    let x = query.find("Usages::x")[0];
    assert_eq!(
        query.source(x.source.as_ref().unwrap()).unwrap(),
        "part x : Definitions::Base;"
    );
    let references = query.references_from(&x.id).unwrap();
    let r = references
        .iter()
        .find(|r| r.kind == RelationshipKind::Typing)
        .unwrap();
    assert!(!query
        .references_at("usages", r.source.start.byte_offset)
        .unwrap()
        .is_empty());
    assert!(query
        .references_at("usages", r.source.end.byte_offset)
        .unwrap()
        .is_empty());
    assert!(matches!(
        query.references_at("definitions", 4),
        Err(QueryError::InvalidSourceRange)
    ));
    assert!(matches!(
        query.diagnostics("unknown"),
        Err(QueryError::UnknownDocument(_))
    ));
    assert!(matches!(
        query.references_from("unknown"),
        Err(QueryError::UnknownElement(_))
    ));
}

#[test]
fn deserialized_snapshots_are_checked_before_indexed_queries() {
    let original = snapshot();
    let mut bad = original.clone();
    bad.elements.push(bad.elements[0].clone());
    assert!(bad.queries().is_err());
    let mut bad = original.clone();
    bad.references[0].owner_id = Some("dangling".into());
    assert!(bad.queries().is_err());
    let mut bad = original.clone();
    bad.references[0].source.start.utf16_offset += 1;
    assert!(matches!(bad.queries(), Err(QueryError::InvalidSourceRange)));
    let mut bad = original.clone();
    bad.connections[0].ends[0].chain.clear();
    assert!(bad.queries().is_err());
    let mut bad = original.clone();
    bad.schema_version += 1;
    assert!(bad.queries().is_err());
    let mut bad = original.clone();
    bad.elements[0].source.as_mut().unwrap().document_id = "absent".into();
    assert!(bad.queries().is_err());
}

#[test]
fn syntax_check_recovers_without_linking_or_loading_libraries() {
    let good = check_syntax(document(
        "single",
        "package Local { part x : ExternalType; }",
    ))
    .unwrap();
    assert!(good.diagnostics.is_empty());
    assert!(good.document.text.contains("ExternalType"));
    let bad = check_syntax(document("broken", "package Local { part x : ; }")).unwrap();
    assert!(bad
        .diagnostics
        .iter()
        .any(|d| matches!(d.stage, Stage::Syntax)));
    assert!(bad
        .diagnostics
        .iter()
        .all(|d| matches!(d.stage, Stage::Syntax | Stage::Structure)));
    let kerml = SourceDocument {
        id: "kerml".into(),
        name: "Example.KERML".into(),
        text: "class C; feature x : C;".into(),
    };
    assert!(check_syntax(kerml).unwrap().diagnostics.is_empty());
}

#[test]
fn input_policy_applies_before_syntax_or_semantic_construction() {
    let mut invalid = document("", "");
    assert!(matches!(
        check_syntax(invalid.clone()),
        Err(AnalysisError::InvalidInput(_))
    ));
    invalid.id = "valid".into();
    invalid.name = "unknown.txt".into();
    assert!(check_syntax(invalid).is_err());
    assert!(matches!(
        check_syntax(document("oversize", &" ".repeat(5_000_001))),
        Err(AnalysisError::InvalidInput(_))
    ));
    let documents = (0..33).map(|i| document(&i.to_string(), "")).collect();
    assert!(matches!(
        Engine::bundled().analyze(
            AnalysisRequest {
                revision: 1,
                documents
            },
            || false
        ),
        Err(AnalysisError::InvalidInput(_))
    ));
}

#[test]
fn deeply_nested_syntax_is_bounded_and_keeps_diagnostic_ranges_valid() {
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let text = "package Nested {".repeat(512) + &"}".repeat(512);
            let report = check_syntax(document("deep", &text)).unwrap();
            assert!(
                !report.diagnostics.is_empty(),
                "Depth limit must be observable"
            );
            assert!(report.diagnostics.iter().all(|d| text
                .get(d.source.start.byte_offset as usize..d.source.end.byte_offset as usize)
                .is_some()));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn completed_snapshot_remains_usable_after_a_later_request_is_cancelled() {
    let old = snapshot();
    let cancelled = Engine::bundled().analyze(
        AnalysisRequest {
            revision: 42,
            documents: vec![],
        },
        || true,
    );
    assert!(matches!(cancelled, Err(AnalysisError::Cancelled)));
    assert_eq!(old.queries().unwrap().find("Usages::x").len(), 1);
}

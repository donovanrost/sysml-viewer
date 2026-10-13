use std::sync::OnceLock;
use sysml_kernel::*;

fn analyze(text: &str) -> Snapshot {
    let text = text.to_owned();
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(move || {
            Engine::bundled()
                .analyze(
                    AnalysisRequest {
                        revision: 31,
                        documents: vec![SourceDocument {
                            id: "imports".into(),
                            name: "imports.sysml".into(),
                            text,
                        }],
                    },
                    || false,
                )
                .unwrap()
        })
        .unwrap()
        .join()
        .unwrap()
}
fn fixture() -> &'static Snapshot {
    static VALUE: OnceLock<Snapshot> = OnceLock::new();
    VALUE.get_or_init(|| {
        analyze(include_str!(
            "../../../tests/conformance/fixtures/imp-020.sysml"
        ))
    })
}
fn imports<'a>(q: &'a SnapshotQuery<'a>, name: &str) -> &'a [ImportedMembership] {
    match q.imported_memberships(&q.find(name)[0].id).unwrap() {
        ImportClosure::Complete { memberships } => memberships,
        other => panic!("{name}: {other:?}"),
    }
}

#[test]
fn imported_alias_identity_and_contextual_visibility_survive_round_trip() {
    let restored: Snapshot =
        serde_json::from_str(&serde_json::to_string(fixture()).unwrap()).unwrap();
    let q = restored.queries().unwrap();
    let source = q.find("ImportSource")[0];
    let owned = q.owned_memberships(&source.id).unwrap();
    let alias = owned.iter().find(|m| m.is_alias).unwrap();
    let target = q.find("ImportSource::Engine")[0];
    assert_eq!(
        alias.member,
        Resolution::Resolved {
            target_id: target.id.clone()
        }
    );
    assert_eq!(target.parent_id.as_deref(), Some(source.id.as_str()));
    let bindings = imports(&q, "ImportConsumer");
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].membership_id, alias.id);
    assert_eq!(bindings[0].visibility, Visibility::Protected);
    assert_eq!(alias.visibility, Visibility::Public);
    assert_eq!(alias.short_name.as_deref(), Some("E"));
    assert_eq!(
        q.membership(&bindings[1].membership_id)
            .unwrap()
            .name
            .as_deref(),
        Some("Engine")
    );
    assert_eq!(bindings[1].visibility, Visibility::Private);
    let reexport = imports(&q, "ImportReexport");
    let downstream = imports(&q, "ImportDownstream");
    assert_eq!(
        reexport
            .iter()
            .map(|b| &b.membership_id)
            .collect::<Vec<_>>(),
        downstream
            .iter()
            .map(|b| &b.membership_id)
            .collect::<Vec<_>>()
    );
    assert!(reexport.iter().all(|b| b.visibility == Visibility::Public));
    assert!(downstream
        .iter()
        .all(|b| b.visibility == Visibility::Private));
    assert!(imports(&q, "ImportEmpty").is_empty());
    assert!(matches!(
        q.imported_memberships("missing"),
        Err(QueryError::UnknownElement(_))
    ));
    let stub = restored
        .elements
        .iter()
        .find(|e| !e.members_available)
        .unwrap();
    assert_eq!(
        q.imported_memberships(&stub.id).unwrap(),
        &ImportClosure::Unknown {
            reason: ImportIssue::Unavailable
        }
    );
}

#[test]
fn malformed_import_transport_is_rejected() {
    for mutation in 0..4 {
        let mut snapshot = fixture().clone();
        let index = snapshot
            .elements
            .iter()
            .position(|e| e.qualified_name.as_deref() == Some("ImportConsumer"))
            .unwrap();
        let ImportClosure::Complete { memberships } =
            &mut snapshot.elements[index].imported_memberships
        else {
            panic!()
        };
        match mutation {
            0 => memberships[0].membership_id = "dangling".into(),
            1 => memberships.push(memberships[0].clone()),
            2 => {
                let id = memberships[0].membership_id.clone();
                snapshot
                    .memberships
                    .iter_mut()
                    .find(|m| m.id == id)
                    .unwrap()
                    .member = Resolution::Unresolved;
            }
            3 => {
                let stub = snapshot
                    .elements
                    .iter_mut()
                    .find(|e| !e.members_available)
                    .unwrap();
                stub.imported_memberships = ImportClosure::Complete {
                    memberships: vec![],
                };
            }
            _ => unreachable!(),
        }
        assert!(snapshot.queries().is_err(), "mutation {mutation}");
    }
}

#[test]
fn unsupported_and_missing_import_providers_are_not_empty_closures() {
    let snapshot = analyze(include_str!(
        "../../../tests/conformance/fixtures/imp-026.sysml"
    ));
    let q = snapshot.queries().unwrap();
    for name in ["MissingImport", "IncompleteImport", "FilterImport"] {
        assert_eq!(
            q.imported_memberships(&q.find(name)[0].id).unwrap(),
            &ImportClosure::Unknown {
                reason: ImportIssue::IncompleteProvider
            },
            "{name}"
        );
    }
    assert!(imports(&q, "FilterFalse").is_empty());
}

#[test]
fn syntax_recovery_cannot_certify_an_empty_import_closure() {
    let snapshot = analyze(concat!(
        include_str!("../../../tests/conformance/fixtures/imp-027.sysml"),
        "\npackage OtherwiseEmpty;"
    ));
    let q = snapshot.queries().unwrap();
    for name in ["RecoveredImport", "OtherwiseEmpty"] {
        assert_eq!(
            q.imported_memberships(&q.find(name)[0].id).unwrap(),
            &ImportClosure::Unknown {
                reason: ImportIssue::InvalidSyntax
            }
        );
    }
}

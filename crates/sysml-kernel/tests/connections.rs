use std::sync::OnceLock;
use sysml_kernel::*;

fn fixture() -> &'static Snapshot {
    static SNAPSHOT: OnceLock<Snapshot> = OnceLock::new();
    SNAPSHOT.get_or_init(|| {
        std::thread::Builder::new()
            .stack_size(ANALYSIS_STACK_BYTES)
            .spawn(|| {
                let documents = [
                    (
                        "conn-004.sysml",
                        include_str!("../../../tests/conformance/fixtures/conn-004.sysml"),
                    ),
                    (
                        "conn-005.kerml",
                        include_str!("../../../tests/conformance/fixtures/conn-005.kerml"),
                    ),
                ]
                .into_iter()
                .map(|(name, text)| SourceDocument {
                    id: name.into(),
                    name: name.into(),
                    text: text.into(),
                })
                .collect();
                Engine::bundled()
                    .analyze(
                        AnalysisRequest {
                            revision: 41,
                            documents,
                        },
                        || false,
                    )
                    .unwrap()
            })
            .unwrap()
            .join()
            .unwrap()
    })
}

fn connection<'a>(snapshot: &'a Snapshot, name: &str) -> &'a Connection {
    let element = snapshot.find(name);
    assert_eq!(element.len(), 1, "{name}");
    snapshot
        .connections
        .iter()
        .find(|c| c.element_id == element[0].id)
        .unwrap()
}

fn check_ends(snapshot: &Snapshot, name: &str, resolved: &[bool]) {
    let connection = connection(snapshot, name);
    assert_eq!(connection.ends.len(), resolved.len(), "{name}");
    for (end, expected) in connection.ends.iter().zip(resolved) {
        assert_eq!(
            matches!(end.resolution, Resolution::Resolved { .. }),
            *expected,
            "{name}: {end:?}"
        );
        if let Resolution::Resolved { target_id } = &end.resolution {
            assert_eq!(end.chain.last(), Some(target_id));
        } else {
            assert!(matches!(end.resolution, Resolution::Unsupported { .. }));
        }
    }
}

#[test]
fn broken_and_ambiguous_ends_do_not_hide_resolved_peers() {
    let snapshot = fixture();
    assert!(
        !snapshot
            .diagnostics
            .iter()
            .any(|d| matches!(d.stage, Stage::Syntax)),
        "{:?}",
        snapshot.diagnostics
    );
    for (name, expected) in [
        ("ScopedEnds::host::missingFirst", vec![false, true]),
        ("ScopedEnds::host::missingLast", vec![true, false]),
        ("ScopedEnds::host::deepMissing", vec![false, true]),
        ("ScopedEnds::host::missingMiddle", vec![false, true]),
        ("ScopedEnds::host::triple", vec![true, false, true]),
        ("ScopedEnds::ambiguousHost::wire", vec![false, true]),
        ("KerMLEnds::broken", vec![false, true]),
        ("KerMLEnds::triple", vec![true, false, true]),
    ] {
        check_ends(snapshot, name, &expected);
    }
    let broken = connection(snapshot, "ScopedEnds::host::deepMissing");
    let names: Vec<_> = broken.ends[0]
        .chain
        .iter()
        .map(|id| {
            snapshot
                .element(id)
                .unwrap()
                .qualified_name
                .as_deref()
                .unwrap()
        })
        .collect();
    assert_eq!(names, ["ScopedEnds::host::a", "ScopedEnds::Branch::leaf"]);
    assert!(snapshot
        .references
        .iter()
        .any(|r| r.spelling == "duplicate" && r.resolution == Resolution::Ambiguous));
}

#[test]
fn unrelated_body_errors_leave_end_targets_queryable_but_end_errors_stay_guarded() {
    let snapshot = fixture();
    for name in [
        "ScopedEnds::host::bodyError",
        "ScopedEnds::host::healthy",
        "KerMLEnds::bodyError",
        "KerMLEnds::healthy",
    ] {
        check_ends(snapshot, name, &[true, true]);
    }
    for name in ["ScopedEnds::host::endBound", "KerMLEnds::bound"] {
        check_ends(snapshot, name, &[false, true]);
    }
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|d| d.message.contains("Missing")));
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|d| d.message.contains("missingBound")));
}

#[test]
fn endpoint_uncertainty_and_partial_chains_survive_json_roundtrip() {
    let snapshot = fixture();
    let bytes = serde_json::to_vec(snapshot).unwrap();
    let decoded: Snapshot = serde_json::from_slice(&bytes).unwrap();
    decoded.queries().unwrap();
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), bytes);
    for name in ["ScopedEnds::host::deepMissing", "KerMLEnds::broken"] {
        check_ends(&decoded, name, &[false, true]);
    }
}

#[test]
fn flow_redefinitions_are_scoped_to_their_end_and_implicit_ends_stay_unknown() {
    let snapshot = fixture();
    check_ends(snapshot, "FlowEnds::pipes::broken", &[false, true]);
    check_ends(snapshot, "FlowEnds::pipes::healthy", &[true, true]);
    let action = snapshot.find("FlowEnds::run")[0];
    let implicit: Vec<_> = snapshot
        .connections
        .iter()
        .filter(|c| {
            snapshot
                .element(&c.element_id)
                .unwrap()
                .parent_id
                .as_deref()
                == Some(action.id.as_str())
        })
        .collect();
    assert_eq!(implicit.len(), 1);
    assert_eq!(implicit[0].ends.len(), 2);
    for end in &implicit[0].ends {
        assert!(end.chain.is_empty());
        assert!(matches!(end.resolution, Resolution::Unsupported { .. }));
    }
}

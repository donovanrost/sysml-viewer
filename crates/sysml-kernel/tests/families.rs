use std::sync::OnceLock;
use sysml_kernel::*;

const SOURCES: &[(&str, &str)] = &[
    (
        "family-kernel-types.kerml",
        include_str!("../../../tests/conformance/fixtures/family-kernel-types.kerml"),
    ),
    (
        "type-005.sysml",
        include_str!("../../../tests/conformance/fixtures/type-005.sysml"),
    ),
    (
        "type-006.sysml",
        include_str!("../../../tests/conformance/fixtures/type-006.sysml"),
    ),
    (
        "type-007.sysml",
        include_str!("../../../tests/conformance/fixtures/type-007.sysml"),
    ),
];

fn fixture() -> &'static Snapshot {
    static SNAPSHOT: OnceLock<Snapshot> = OnceLock::new();
    SNAPSHOT.get_or_init(|| {
        std::thread::Builder::new()
            .stack_size(ANALYSIS_STACK_BYTES)
            .spawn(|| {
                Engine::bundled()
                    .analyze(
                        AnalysisRequest {
                            revision: 43,
                            documents: SOURCES
                                .iter()
                                .map(|(name, text)| SourceDocument {
                                    id: (*name).into(),
                                    name: (*name).into(),
                                    text: (*text).into(),
                                })
                                .collect(),
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

fn findings<'a>(snapshot: &'a Snapshot, owner: &str) -> Vec<&'a Diagnostic> {
    let elements = snapshot.find(owner);
    assert_eq!(elements.len(), 1, "{owner}");
    let source = elements[0].source.as_ref().unwrap();
    snapshot
        .diagnostics
        .iter()
        .filter(|d| {
            d.source.document_id == source.document_id
                && d.source.start.byte_offset >= source.start.byte_offset
                && d.source.end.byte_offset <= source.end.byte_offset
        })
        .collect()
}

#[test]
fn kernel_classifier_families_accept_states_and_exhibits_typed_by_behavior() {
    let snapshot = fixture();
    assert!(snapshot
        .diagnostics
        .iter()
        .all(|d| !matches!(d.stage, Stage::Syntax | Stage::Structure)));
    for file in ["family-kernel-types.kerml", "type-005.sysml"] {
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|d| d.source.document_id != file),
            "{:?}",
            snapshot.diagnostics
        );
    }
    let query = snapshot.queries().unwrap();
    for (name, expected) in [
        ("direct", vec!["FamilyKernelTypes::BehaviorBase"]),
        (
            "multiple",
            vec![
                "FamilyKernelTypes::BehaviorBase",
                "FamilyKernelTypes::OtherBehavior",
            ],
        ),
        (
            "related",
            vec![
                "FamilyKernelTypes::BehaviorBase",
                "FamilyKernelTypes::BehaviorChild",
            ],
        ),
        ("shown", vec!["FamilyKernelTypes::BehaviorChild"]),
        ("actionTyped", vec!["KernelFamilyTyping::LocalAction"]),
    ] {
        let e = query.find(&format!("KernelFamilyTyping::{name}"))[0];
        assert_eq!(
            e.metaclass,
            if name == "shown" {
                "ExhibitStateUsage"
            } else {
                "StateUsage"
            }
        );
        let mut targets: Vec<_> = query
            .references_from(&e.id)
            .unwrap()
            .iter()
            .filter(|r| r.kind == RelationshipKind::Typing)
            .map(|r| match &r.resolution {
                Resolution::Resolved { target_id } => query
                    .element(target_id)
                    .unwrap()
                    .qualified_name
                    .as_deref()
                    .unwrap(),
                other => panic!("{name}: {other:?}"),
            })
            .collect();
        targets.sort_unstable();
        let mut expected = expected;
        expected.sort_unstable();
        assert_eq!(targets, expected);
    }
}

#[test]
fn concern_rejections_are_local_and_redundant_specific_types_stay_valid() {
    let snapshot = fixture();
    let query = snapshot.queries().unwrap();
    for name in ["invalid", "mixed"] {
        let errors = findings(snapshot, &format!("ConcernTyping::{name}"));
        assert_eq!(errors.len(), 1, "{name}: {errors:?}");
        assert!(matches!(errors[0].stage, Stage::Semantic));
        assert_eq!(errors[0].severity, Severity::Error);
        assert!(errors[0]
            .message
            .contains("ConcernUsage must be typed by ConcernDefinition"));
        assert_eq!(query.source(&errors[0].source).unwrap(), "General");
    }
    let errors = findings(snapshot, "ConcernTyping::multiple");
    assert_eq!(errors.len(), 1);
    assert!(errors[0].message.contains("exactly one non-redundant type"));
    for name in ["asRequirement", "valid", "inherited", "redundant"] {
        assert!(findings(snapshot, &format!("ConcernTyping::{name}")).is_empty());
    }
}

#[test]
fn incompatible_and_unresolved_state_types_remain_distinct_after_round_trip() {
    let bytes = serde_json::to_vec(fixture()).unwrap();
    let decoded: Snapshot = serde_json::from_slice(&bytes).unwrap();
    let query = decoded.queries().unwrap();
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), bytes);
    for name in ["bad", "badExhibit", "mixed"] {
        let errors = findings(&decoded, &format!("StateTypingGuards::{name}"));
        assert_eq!(errors.len(), 1, "{name}: {errors:?}");
        assert!(matches!(errors[0].stage, Stage::Semantic));
        assert_eq!(errors[0].severity, Severity::Error);
        assert!(errors[0].message.contains("must be typed by Behavior"));
        assert_eq!(query.source(&errors[0].source).unwrap(), "Data");
        let e = query.find(&format!("StateTypingGuards::{name}"))[0];
        assert!(query
            .references_from(&e.id)
            .unwrap()
            .iter()
            .filter(|r| r.kind == RelationshipKind::Typing)
            .all(|r| matches!(r.resolution, Resolution::Resolved { .. })));
    }
    assert!(findings(&decoded, "StateTypingGuards::good").is_empty());
    let missing = query.find("StateTypingGuards::missing")[0];
    assert!(query
        .references_from(&missing.id)
        .unwrap()
        .iter()
        .any(|r| r.resolution == Resolution::Unresolved));
    let errors = findings(&decoded, "StateTypingGuards::missing");
    assert!(errors.iter().any(|d| matches!(d.stage, Stage::Resolution)));
    // Missing classifiers also make the facade's inherited-member query
    // incomplete. That capability finding is separate from type incompatibility.
    assert!(!missing.inheritance_complete);
    assert!(errors.iter().all(|d| !matches!(d.stage, Stage::Semantic)));
}

#[test]
fn syntax_checking_does_not_mistake_incompatible_types_for_invalid_grammar() {
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(|| {
            for (name, text) in SOURCES {
                let report = check_syntax(SourceDocument {
                    id: (*name).into(),
                    name: (*name).into(),
                    text: (*text).into(),
                })
                .unwrap();
                assert!(report.diagnostics.is_empty(), "{name}: {report:?}");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

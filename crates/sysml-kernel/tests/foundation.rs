use std::sync::OnceLock;
use sysml_kernel::*;

fn analyze(documents: Vec<SourceDocument>) -> Snapshot {
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(move || {
            Engine::bundled()
                .analyze(
                    AnalysisRequest {
                        revision: 23,
                        documents,
                    },
                    || false,
                )
                .unwrap()
        })
        .unwrap()
        .join()
        .unwrap()
}
fn document(name: &str, text: &str) -> SourceDocument {
    SourceDocument {
        id: name.into(),
        name: name.into(),
        text: text.into(),
    }
}

fn provider_fixture() -> &'static Snapshot {
    static VALUE: OnceLock<Snapshot> = OnceLock::new();
    VALUE.get_or_init(|| {
        analyze(vec![
            document(
                "providers.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-011.sysml"),
            ),
            document(
                "guards.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-012.sysml"),
            ),
        ])
    })
}

#[test]
fn package_attribute_providers_retain_exact_bounds_through_queries_and_json() {
    let snapshot: Snapshot =
        serde_json::from_str(&serde_json::to_string(provider_fixture()).unwrap()).unwrap();
    let q = snapshot.queries().unwrap();
    for (name, lower, upper, display) in [
        ("viaAlias", "3", "3", "[3]"),
        ("chain", "3", "3", "[3]"),
        ("ranged", "3", "6", "[3..6]"),
        (
            "exactLarge",
            "9007199254740993",
            "9007199254740993",
            "[9007199254740993]",
        ),
        ("empty", "0", "0", "[0]"),
    ] {
        let element = q.find(&format!("ProviderBounds::{name}"))[0];
        assert_eq!(
            q.cardinality(&element.id).unwrap(),
            &Cardinality::Evaluated {
                lower: lower.into(),
                upper: Some(upper.into()),
            },
            "{name}"
        );
        assert_eq!(element.declared_multiplicity.as_deref(), Some(display));
    }
}

#[test]
fn incomplete_providers_never_become_default_or_empty_intervals() {
    let q = provider_fixture().queries().unwrap();
    for (name, reason) in [
        ("unvaluedBound", CardinalityIssue::IncompleteProvider),
        ("cycleBound", CardinalityIssue::UnsupportedBound),
        ("variationBound", CardinalityIssue::UnsupportedBound),
        ("featuredBound", CardinalityIssue::IncompleteProvider),
    ] {
        let element = q.find(&format!("ProviderGuards::{name}"))[0];
        assert_eq!(
            q.cardinality(&element.id).unwrap(),
            &Cardinality::Unknown { reason },
            "{name}"
        );
        assert_eq!(element.declared_multiplicity, None);
    }
}
fn fixture() -> &'static Snapshot {
    static VALUE: OnceLock<Snapshot> = OnceLock::new();
    VALUE.get_or_init(|| {
        analyze(vec![
            document(
                "aliases.sysml",
                include_str!("../../../tests/conformance/fixtures/imp-008.sysml"),
            ),
            document(
                "inherited.sysml",
                include_str!("../../../tests/conformance/fixtures/imp-014.sysml"),
            ),
            document(
                "markers.sysml",
                "package MembershipMarkers { action def Steps { action work; first work; } }",
            ),
            document(
                "bounds.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-005.sysml"),
            ),
        ])
    })
}

#[test]
fn aliases_keep_binding_identity_names_and_target_ownership() {
    let q = fixture().queries().unwrap();
    let scope = q.find("Bindings")[0];
    let target = q.find("Bindings::Device")[0];
    let members = q.owned_memberships(&scope.id).unwrap();
    let aliases: Vec<_> = members.iter().filter(|m| m.is_alias).collect();
    assert_eq!(aliases.len(), 3);
    for alias in &aliases {
        assert_ne!(alias.id, target.id);
        assert_eq!(
            alias.member,
            Resolution::Resolved {
                target_id: target.id.clone()
            }
        );
        assert_eq!(alias.namespace_id, scope.id);
    }
    assert_eq!(target.parent_id.as_deref(), Some(scope.id.as_str()));
    assert_eq!(
        q.owned_members(&scope.id)
            .unwrap()
            .iter()
            .filter(|e| e.id == target.id)
            .count(),
        1
    );
    let short = aliases
        .iter()
        .find(|m| m.short_name.as_deref() == Some("OnlyShort"))
        .unwrap();
    assert_eq!(short.name, None);
    assert!(
        q.find("Bindings::Copy").is_empty(),
        "Canonical lookup is not alias resolution"
    );
}

#[test]
fn inherited_aliases_retain_their_home_membership_identity_and_visibility() {
    let q = fixture().queries().unwrap();
    let parent = q.find("AliasHeritage::Parent")[0];
    let child = q.find("AliasHeritage::Child")[0];
    let owned = q.owned_memberships(&parent.id).unwrap();
    let protected = owned
        .iter()
        .find(|m| m.name.as_deref() == Some("renamed"))
        .unwrap();
    assert_eq!(protected.visibility, Visibility::Protected);
    let inherited = q.inherited_memberships(&child.id).unwrap();
    assert!(inherited.inheritance_complete);
    let inherited = inherited.memberships;
    assert!(inherited
        .iter()
        .any(|m| m.id == protected.id && m.namespace_id == parent.id));
    assert!(!inherited
        .iter()
        .any(|m| m.name.as_deref() == Some("hidden")));
    assert!(matches!(
        q.membership("unknown"),
        Err(QueryError::UnknownMembership(_))
    ));
    let library = fixture().elements.iter().find(|e| e.library).unwrap();
    assert!(matches!(
        q.owned_memberships(&library.id),
        Err(QueryError::MembersUnavailable(_))
    ));
}

#[test]
fn exact_cardinality_survives_json_round_trip_and_checked_queries() {
    let restored: Snapshot =
        serde_json::from_str(&serde_json::to_string(fixture()).unwrap()).unwrap();
    let q = restored.queries().unwrap();
    let element = q.find("ExactBounds::Assembly::maximum")[0];
    assert_eq!(
        q.cardinality(&element.id).unwrap(),
        &Cardinality::Evaluated {
            lower: i128::MAX.to_string(),
            upper: Some(i128::MAX.to_string())
        }
    );
    assert_eq!(
        element.declared_multiplicity,
        Some(format!("[{}]", i128::MAX))
    );
}

#[test]
fn malformed_membership_and_cardinality_transport_is_rejected() {
    for mutation in 0..6 {
        let mut value = fixture().clone();
        let element = value
            .elements
            .iter_mut()
            .find(|e| e.qualified_name.as_deref() == Some("Bindings"))
            .unwrap();
        match mutation {
            0 => element.owned_memberships.push("missing".into()),
            1 => value.memberships[0].namespace_id = "missing".into(),
            2 => value.memberships.push(value.memberships[0].clone()),
            3 => {
                element.cardinality = Cardinality::Evaluated {
                    lower: "01".into(),
                    upper: None,
                }
            }
            4 => {
                element.cardinality = Cardinality::Evaluated {
                    lower: "4".into(),
                    upper: Some("3".into()),
                }
            }
            _ => {
                element.cardinality = Cardinality::Evaluated {
                    lower: "-1".into(),
                    upper: None,
                }
            }
        }
        assert!(value.queries().is_err(), "Mutation {mutation} was accepted");
    }
}

#[test]
fn recovered_syntax_cannot_supply_defaults_to_a_clean_dependent_file() {
    let s = analyze(vec![
        document(
            "broken.sysml",
            "package BrokenBounds { part def Assembly { part malformed [-1]; } }",
        ),
        document("dependent.sysml", "part consumer : BrokenBounds::Assembly;"),
    ]);
    let q = s.queries().unwrap();
    assert_eq!(
        q.cardinality(&q.find("consumer")[0].id).unwrap(),
        &Cardinality::Unknown {
            reason: CardinalityIssue::InvalidSyntax
        }
    );
}

#[test]
fn every_pinned_standard_library_source_passes_syntax_and_structure() {
    // This exercises all 94 library files, but is not an exhaustive grammar or
    // semantic conformance claim. Include the exact build-time embedded bytes.
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let sources: Vec<serde_json::Value> =
                serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/library.json")))
                    .unwrap();
            assert_eq!(sources.len(), 94);
            let failures: Vec<_> = sources
                .into_iter()
                .filter_map(|source| {
                    let report = check_syntax(document(
                        source["name"].as_str().unwrap(),
                        source["text"].as_str().unwrap(),
                    ))
                    .unwrap();
                    (!report.diagnostics.is_empty())
                        .then_some((report.document.name, report.diagnostics))
                })
                .collect();
            assert!(failures.is_empty(), "{failures:#?}");
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn top_level_import_visibility_checks_both_dialects_kinds_and_nesting() {
    // The Import predicate depends on root ownership and visibility. Exhaust
    // those partitions for both concrete dialects and both import subclasses.
    // Other SysML rules (e.g. requiring an explicit import visibility) can also
    // diagnose a case; this assertion concerns only the named KerML rule.
    for dialect in ["sysml", "kerml"] {
        for imported in ["ScalarValues::*", "ScalarValues::Real"] {
            for visibility in ["private", "protected", "public", ""] {
                for root in [true, false] {
                    let import = format!("{visibility} import {imported};");
                    let text = if root {
                        import
                    } else {
                        format!("package Nested {{ {import} }}")
                    };
                    let report =
                        check_syntax(document(&format!("partition.{dialect}"), &text)).unwrap();
                    let findings: Vec<_> = report
                        .diagnostics
                        .iter()
                        .filter(|d| d.message.contains("validateImportTopLevelVisibility"))
                        .collect();
                    assert_eq!(
                        !findings.is_empty(),
                        root && visibility != "private",
                        "{dialect}: {text}: {:?}",
                        report.diagnostics
                    );
                }
            }
        }
    }
}

#[test]
fn anonymous_reference_memberships_do_not_acquire_the_target_name() {
    let q = fixture().queries().unwrap();
    let owner = q.find("MembershipMarkers::Steps")[0];
    let target = q.find("MembershipMarkers::Steps::work")[0];
    let members = q.owned_memberships(&owner.id).unwrap();
    let marker = members
        .iter()
        .find(|m| m.metaclass == "Membership" && !m.is_alias)
        .unwrap();
    assert_eq!(marker.name, None);
    assert_eq!(marker.short_name, None);
    assert_eq!(
        marker.member,
        Resolution::Resolved {
            target_id: target.id.clone()
        }
    );
}

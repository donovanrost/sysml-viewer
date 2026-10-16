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

fn arithmetic_fixture() -> &'static Snapshot {
    static VALUE: OnceLock<Snapshot> = OnceLock::new();
    VALUE.get_or_init(|| {
        analyze(vec![
            document(
                "formulas.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-003.sysml"),
            ),
            document(
                "arithmetic.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-013.sysml"),
            ),
            document(
                "invalid-arithmetic.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-014.sysml"),
            ),
        ])
    })
}

#[test]
fn conditional_bounds_select_only_the_chosen_expression() {
    let snapshot = analyze(vec![document(
        "conditional.sysml",
        include_str!("../../../tests/conformance/fixtures/mult-017.sysml"),
    )]);
    assert!(
        snapshot.diagnostics.is_empty(),
        "{:?}",
        snapshot.diagnostics
    );
    let q = snapshot.queries().unwrap();
    for (name, value) in [
        ("trueBranch", "3"),
        ("falseBranch", "4"),
        ("nestedBranch", "5"),
        ("arithmetic", "10"),
        ("skippedDivision", "6"),
        ("skippedUnbound", "7"),
        ("exactLarge", "9007199254740993"),
        ("nestedCondition", "8"),
    ] {
        let e = q.find(&format!("ConditionalProviders::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Evaluated {
                lower: value.into(),
                upper: Some(value.into())
            },
            "{name}"
        );
        assert_eq!(
            e.declared_multiplicity.as_deref(),
            Some(format!("[{value}]").as_str())
        );
    }
    let e = q.find("ConditionalProviders::range")[0];
    assert_eq!(
        q.cardinality(&e.id).unwrap(),
        &Cardinality::Evaluated {
            lower: "3".into(),
            upper: Some("4".into())
        }
    );
}

#[test]
fn arithmetic_bounds_use_complete_bindings_even_alongside_unadmitted_calls() {
    let q = arithmetic_fixture().queries().unwrap();
    for (name, value) in [
        ("add", "10"),
        ("subtract", "5"),
        ("multiply", "16"),
        ("divide", "4"),
        ("modulo", "2"),
        ("power", "8"),
        ("compound", "30"),
        ("unary", "1"),
        ("exactLarge", "9007199254740993"),
    ] {
        let e = q.find(&format!("ArithmeticBounds::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Evaluated {
                lower: value.into(),
                upper: Some(value.into()),
            },
            "{name}"
        );
        assert_eq!(
            e.declared_multiplicity.as_deref(),
            Some(format!("[{value}]").as_str())
        );
    }
    let e = q.find("FormulaBounds::Assembly::ranged")[0];
    assert_eq!(
        q.cardinality(&e.id).unwrap(),
        &Cardinality::Evaluated {
            lower: "3".into(),
            upper: Some("6".into()),
        }
    );
}

#[test]
fn conditional_bounds_do_not_coerce_unavailable_conditions_or_invalid_selected_values() {
    let snapshot = analyze(vec![document(
        "conditional-guards.sysml",
        include_str!("../../../tests/conformance/fixtures/mult-018.sysml"),
    )]);
    let q = snapshot.queries().unwrap();
    for name in [
        "unavailable",
        "wrongType",
        "unbound",
        "division",
        "belowZero",
        "fraction",
        "overflow",
        "recursive",
        "ineligible",
    ] {
        let e = q.find(&format!("ConditionalGuards::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Unknown {
                reason: if matches!(name, "unavailable" | "unbound") {
                    CardinalityIssue::IncompleteProvider
                } else {
                    CardinalityIssue::UnsupportedBound
                },
            },
            "{name}"
        );
        assert_eq!(e.declared_multiplicity, None);
    }
}

#[test]
fn arithmetic_failures_never_become_successful_numeric_bounds() {
    let q = arithmetic_fixture().queries().unwrap();
    for name in [
        "division",
        "fraction",
        "belowZero",
        "overflow",
        "cycle",
        "call",
    ] {
        let e = q.find(&format!("ArithmeticGuards::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Unknown {
                reason: CardinalityIssue::UnsupportedBound,
            },
            "{name}"
        );
        assert_eq!(e.declared_multiplicity, None);
    }
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

#[test]
fn compound_ranges_preserve_exact_values_and_unknown_endpoints_after_round_trip() {
    for with_missing_base in [false, true] {
        let mut documents = vec![
            document(
                "ranges.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-022.sysml"),
            ),
            document(
                "guards.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-023.sysml"),
            ),
        ];
        if with_missing_base {
            documents.push(document(
                "missing.sysml",
                "package BrokenRangeContext { part def Base :> MissingBase; }",
            ));
        }
        let original = analyze(documents);
        let restored: Snapshot =
            serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
        let q = restored.queries().unwrap();
        for (name, lower, upper) in [
            ("exact", "5", "5"),
            ("chosen", "10", "10"),
            ("range", "5", "10"),
            ("viaAlias", "5", "10"),
            ("repeated", "10", "10"),
            ("repeatedLower", "5", "5"),
        ] {
            let e = q.find(&format!("CompoundRanges::{name}"))[0];
            assert_eq!(
                q.cardinality(&e.id).unwrap(),
                &Cardinality::Evaluated {
                    lower: lower.into(),
                    upper: Some(upper.into())
                },
                "{name}"
            );
        }
        for name in ["incomplete", "fraction", "nonBoolean", "reversed"] {
            let e = q.find(&format!("CompoundRangeGuards::{name}"))[0];
            assert_eq!(
                q.cardinality(&e.id).unwrap(),
                &Cardinality::Unknown {
                    reason: if name == "incomplete" {
                        CardinalityIssue::IncompleteProvider
                    } else {
                        CardinalityIssue::UnsupportedBound
                    }
                },
                "{name}"
            );
        }
    }
}

#[test]
fn ordered_predicates_preserve_exact_values_and_guards_after_round_trip() {
    let original = analyze(vec![
        document(
            "ordered.sysml",
            include_str!("../../../tests/conformance/fixtures/mult-024.sysml"),
        ),
        document(
            "guards.sysml",
            include_str!("../../../tests/conformance/fixtures/mult-025.sysml"),
        ),
    ]);
    assert!(!original
        .diagnostics
        .iter()
        .any(|d| matches!(d.stage, Stage::Syntax)));
    let restored: Snapshot =
        serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
    let q = restored.queries().unwrap();
    for (name, value) in [
        ("ltLess", "7"),
        ("ltEqual", "9"),
        ("ltGreater", "9"),
        ("leLess", "7"),
        ("leEqual", "7"),
        ("leGreater", "9"),
        ("gtLess", "9"),
        ("gtEqual", "9"),
        ("gtGreater", "7"),
        ("geLess", "9"),
        ("geEqual", "7"),
        ("geGreater", "7"),
        ("large", "11"),
        ("largeReverse", "12"),
        ("limit", "13"),
        ("negative", "14"),
        ("rational", "15"),
        ("rationalEqual", "16"),
        ("rationalMixed", "17"),
        ("rationalClose", "18"),
        ("arithmetic", "19"),
        ("viaAlias", "20"),
        ("nested", "21"),
        ("lazy", "22"),
        ("wideArithmetic", "23"),
    ] {
        let e = q.find(&format!("OrderedComparisons::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Evaluated {
                lower: value.into(),
                upper: Some(value.into())
            },
            "{name}"
        );
    }
    for name in [
        "booleanLeft",
        "booleanRight",
        "real",
        "infinity",
        "division",
        "excessive",
        "booleanBound",
        "missingOperand",
        "selectedFraction",
    ] {
        let e = q.find(&format!("OrderedComparisonGuards::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Unknown {
                reason: if name == "missingOperand" {
                    CardinalityIssue::IncompleteProvider
                } else {
                    CardinalityIssue::UnsupportedBound
                }
            },
            "{name}"
        );
    }
}

fn predicate_snapshot() -> &'static Snapshot {
    static VALUE: OnceLock<Snapshot> = OnceLock::new();
    VALUE.get_or_init(|| {
        let original = analyze(vec![
            document(
                "equality.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-026.sysml"),
            ),
            document(
                "logic.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-027.sysml"),
            ),
            document(
                "guards.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-028.sysml"),
            ),
        ]);
        assert!(!original
            .diagnostics
            .iter()
            .any(|d| matches!(d.stage, Stage::Syntax)));
        serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap()
    })
}
#[test]
fn equality_and_logical_predicates_preserve_exact_truth_tables_after_round_trip() {
    let q = predicate_snapshot().queries().unwrap();
    for (name, value) in [
        ("PredicateGuards::stringEquality", "7"),
        ("PredicateGuards::nullEquality", "7"),
        ("EqualityPredicates::eqEqual", "7"),
        ("EqualityPredicates::eqUnequal", "9"),
        ("EqualityPredicates::eqTrue", "7"),
        ("EqualityPredicates::eqFalse", "9"),
        ("EqualityPredicates::eqRational", "7"),
        ("EqualityPredicates::eqLarge", "9"),
        ("EqualityPredicates::neEqual", "9"),
        ("EqualityPredicates::neUnequal", "7"),
        ("EqualityPredicates::neTrue", "9"),
        ("EqualityPredicates::neFalse", "7"),
        ("EqualityPredicates::neRational", "9"),
        ("EqualityPredicates::neLarge", "7"),
        ("EqualityPredicates::sameEqual", "7"),
        ("EqualityPredicates::sameUnequal", "9"),
        ("EqualityPredicates::sameTrue", "7"),
        ("EqualityPredicates::sameFalse", "9"),
        ("EqualityPredicates::sameRational", "7"),
        ("EqualityPredicates::sameLarge", "9"),
        ("EqualityPredicates::differentEqual", "9"),
        ("EqualityPredicates::differentUnequal", "7"),
        ("EqualityPredicates::differentTrue", "9"),
        ("EqualityPredicates::differentFalse", "7"),
        ("EqualityPredicates::differentRational", "9"),
        ("EqualityPredicates::differentLarge", "7"),
        ("EqualityPredicates::integerRational", "7"),
        ("EqualityPredicates::wideExact", "7"),
        ("EqualityPredicates::closeRational", "9"),
        ("EqualityPredicates::signed", "7"),
        ("EqualityPredicates::viaAlias", "7"),
        ("LogicalPredicates::negateTrue", "9"),
        ("LogicalPredicates::negateFalse", "7"),
        ("LogicalPredicates::conjunction00", "9"),
        ("LogicalPredicates::conjunction01", "9"),
        ("LogicalPredicates::conjunction10", "9"),
        ("LogicalPredicates::conjunction11", "7"),
        ("LogicalPredicates::disjunction00", "9"),
        ("LogicalPredicates::disjunction01", "7"),
        ("LogicalPredicates::disjunction10", "7"),
        ("LogicalPredicates::disjunction11", "7"),
        ("LogicalPredicates::exclusive00", "9"),
        ("LogicalPredicates::exclusive01", "7"),
        ("LogicalPredicates::exclusive10", "7"),
        ("LogicalPredicates::exclusive11", "9"),
        ("LogicalPredicates::and00", "9"),
        ("LogicalPredicates::and01", "9"),
        ("LogicalPredicates::and10", "9"),
        ("LogicalPredicates::and11", "7"),
        ("LogicalPredicates::or00", "9"),
        ("LogicalPredicates::or01", "7"),
        ("LogicalPredicates::or10", "7"),
        ("LogicalPredicates::or11", "7"),
        ("LogicalPredicates::implies00", "7"),
        ("LogicalPredicates::implies01", "7"),
        ("LogicalPredicates::implies10", "9"),
        ("LogicalPredicates::implies11", "7"),
        ("LogicalPredicates::skipAnd", "9"),
        ("LogicalPredicates::skipOr", "7"),
        ("LogicalPredicates::skipImplies", "7"),
        ("LogicalPredicates::skipAndDivision", "9"),
        ("LogicalPredicates::skipOrDivision", "7"),
        ("LogicalPredicates::skipImpliesDivision", "7"),
        ("LogicalPredicates::nested", "7"),
        ("LogicalPredicates::combined", "7"),
        ("LogicalPredicates::viaAlias", "7"),
    ] {
        let e = q.find(name)[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Evaluated {
                lower: value.into(),
                upper: Some(value.into()),
            },
            "{name}"
        );
    }
}
#[test]
fn predicate_guards_preserve_unknown_for_required_or_unsupported_values() {
    let q = predicate_snapshot().queries().unwrap();
    for (name, reason) in [
        ("mixedEquality", CardinalityIssue::UnsupportedBound),
        ("mixedInequality", CardinalityIssue::UnsupportedBound),
        ("mixedSame", CardinalityIssue::UnsupportedBound),
        ("mixedNotSame", CardinalityIssue::UnsupportedBound),
        ("realEquality", CardinalityIssue::UnsupportedBound),
        ("infinityEquality", CardinalityIssue::UnsupportedBound),
        ("missingEquality", CardinalityIssue::IncompleteProvider),
        ("undefinedEquality", CardinalityIssue::UnsupportedBound),
        ("excessiveEquality", CardinalityIssue::UnsupportedBound),
        ("notNumber", CardinalityIssue::UnsupportedBound),
        ("andNumber", CardinalityIssue::UnsupportedBound),
        ("orNumber", CardinalityIssue::UnsupportedBound),
        ("xorNumber", CardinalityIssue::UnsupportedBound),
        ("eagerAnd", CardinalityIssue::IncompleteProvider),
        ("eagerOr", CardinalityIssue::IncompleteProvider),
        ("eagerXor", CardinalityIssue::IncompleteProvider),
        ("neededAnd", CardinalityIssue::IncompleteProvider),
        ("neededOr", CardinalityIssue::IncompleteProvider),
        ("neededImplies", CardinalityIssue::IncompleteProvider),
        ("badFirst", CardinalityIssue::UnsupportedBound),
        ("badSecond", CardinalityIssue::UnsupportedBound),
        ("eagerDivision", CardinalityIssue::UnsupportedBound),
        ("selectedDivision", CardinalityIssue::UnsupportedBound),
        ("recursive", CardinalityIssue::UnsupportedBound),
        ("booleanBound", CardinalityIssue::UnsupportedBound),
        ("logicalBound", CardinalityIssue::UnsupportedBound),
    ] {
        let e = q.find(&format!("PredicateGuards::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Unknown { reason },
            "{name}"
        );
    }
}

fn coalescing_snapshot() -> &'static Snapshot {
    static VALUE: OnceLock<Snapshot> = OnceLock::new();
    VALUE.get_or_init(|| {
        let original = analyze(vec![
            document(
                "coalescing.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-029.sysml"),
            ),
            document(
                "guards.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-030.sysml"),
            ),
        ]);
        assert!(!original
            .diagnostics
            .iter()
            .any(|d| matches!(d.stage, Stage::Syntax)));
        serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap()
    })
}
#[test]
fn coalescing_preserves_known_empty_and_present_values_after_round_trip() {
    let q = coalescing_snapshot().queries().unwrap();
    for (name, value) in [
        ("literalNull", "7"),
        ("emptyParentheses", "7"),
        ("presentInteger", "3"),
        ("presentZero", "0"),
        ("falsePresent", "9"),
        ("truePresent", "7"),
        ("nullBoolean", "7"),
        ("rationalPresent", "7"),
        ("viaAlias", "7"),
        ("viaValueAlias", "3"),
        ("nestedEmpty", "7"),
        ("nestedFallback", "7"),
        ("conditionalEmpty", "7"),
        ("conditionalPresent", "3"),
        ("skippedMissing", "3"),
        ("skippedDivision", "3"),
        ("skippedOverflow", "3"),
        ("skippedReal", "3"),
        ("skippedNull", "3"),
        ("arithmeticFallback", "7"),
        ("composedArithmetic", "7"),
        ("composedLogical", "7"),
        ("largeExact", "9007199254740993"),
        ("conditionalSkip", "7"),
    ] {
        let e = q.find(&format!("CoalescingValues::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Evaluated {
                lower: value.into(),
                upper: Some(value.into()),
            },
            "{name}"
        );
    }
}
#[test]
fn coalescing_never_converts_unknown_or_invalid_values_to_empty() {
    let q = coalescing_snapshot().queries().unwrap();
    let e = q.find("CoalescingGuards::emptyEquality")[0];
    assert_eq!(
        q.cardinality(&e.id).unwrap(),
        &Cardinality::Evaluated {
            lower: "7".into(),
            upper: Some("7".into())
        }
    );
    for (name, reason) in [
        ("overflowEndpoint", CardinalityIssue::UnsupportedBound),
        ("underflowFirst", CardinalityIssue::UnsupportedBound),
        ("irrationalFirst", CardinalityIssue::UnsupportedBound),
        ("missingFirst", CardinalityIssue::IncompleteProvider),
        ("selectedMissing", CardinalityIssue::IncompleteProvider),
        ("undefinedFirst", CardinalityIssue::UnsupportedBound),
        ("selectedDivision", CardinalityIssue::UnsupportedBound),
        ("overflowFirst", CardinalityIssue::UnsupportedBound),
        ("bothEmpty", CardinalityIssue::UnsupportedBound),
        ("bareEmpty", CardinalityIssue::UnsupportedBound),
        ("booleanEndpoint", CardinalityIssue::UnsupportedBound),
        ("fractionalEndpoint", CardinalityIssue::UnsupportedBound),
        ("negativeEndpoint", CardinalityIssue::UnsupportedBound),
        ("realFirst", CardinalityIssue::UnsupportedBound),
        ("selectedReal", CardinalityIssue::UnsupportedBound),
        ("stringFirst", CardinalityIssue::UnsupportedBound),
        ("sequenceFirst", CardinalityIssue::UnsupportedBound),
        ("selectedSequence", CardinalityIssue::UnsupportedBound),
        ("recursiveFirst", CardinalityIssue::UnsupportedBound),
        ("recursiveSkipped", CardinalityIssue::UnsupportedBound),
        ("emptyUnaryPlus", CardinalityIssue::UnsupportedBound),
        ("emptyArithmetic", CardinalityIssue::UnsupportedBound),
        ("emptyCondition", CardinalityIssue::UnsupportedBound),
    ] {
        let e = q.find(&format!("CoalescingGuards::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Unknown { reason },
            "{name}"
        );
    }
}

fn empty_comparison_snapshot() -> &'static Snapshot {
    static VALUE: OnceLock<Snapshot> = OnceLock::new();
    VALUE.get_or_init(|| {
        let original = analyze(vec![
            document(
                "empty-comparison.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-031.sysml"),
            ),
            document(
                "guards.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-032.sysml"),
            ),
        ]);
        assert!(!original
            .diagnostics
            .iter()
            .any(|d| matches!(d.stage, Stage::Syntax)));
        serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap()
    })
}
#[test]
fn empty_comparisons_preserve_presence_and_identity_after_round_trip() {
    let q = empty_comparison_snapshot().queries().unwrap();
    for (name, value) in [
        ("eqNull", "7"),
        ("eqParentheses", "7"),
        ("eqZeroRight", "9"),
        ("eqZeroLeft", "9"),
        ("eqFalseRight", "9"),
        ("eqFalseLeft", "9"),
        ("eqRational", "9"),
        ("neNull", "9"),
        ("neParentheses", "9"),
        ("neZeroRight", "7"),
        ("neZeroLeft", "7"),
        ("neFalseRight", "7"),
        ("neFalseLeft", "7"),
        ("neRational", "7"),
        ("sameNull", "7"),
        ("sameParentheses", "7"),
        ("sameZeroRight", "9"),
        ("sameZeroLeft", "9"),
        ("sameFalseRight", "9"),
        ("sameFalseLeft", "9"),
        ("sameRational", "9"),
        ("differentNull", "9"),
        ("differentParentheses", "9"),
        ("differentZeroRight", "7"),
        ("differentZeroLeft", "7"),
        ("differentFalseRight", "7"),
        ("differentFalseLeft", "7"),
        ("differentRational", "7"),
        ("rationalLeft", "7"),
        ("aliasEmpty", "7"),
        ("aliasPresent", "9"),
        ("conditionalEmpty", "7"),
        ("coalescedEmpty", "7"),
        ("coalescedPresent", "7"),
        ("logicalSkip", "7"),
        ("fallbackFalse", "9"),
    ] {
        let e = q.find(&format!("EmptyComparisons::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Evaluated {
                lower: value.into(),
                upper: Some(value.into()),
            },
            "{name}"
        );
    }
}
#[test]
fn empty_comparisons_require_both_values_and_preserve_domain_guards() {
    let q = empty_comparison_snapshot().queries().unwrap();
    let e = q.find("EmptyComparisonGuards::string")[0];
    assert_eq!(
        q.cardinality(&e.id).unwrap(),
        &Cardinality::Evaluated {
            lower: "7".into(),
            upper: Some("7".into())
        }
    );
    for (name, reason) in [
        ("eqMissingRight", CardinalityIssue::IncompleteProvider),
        ("eqMissingLeft", CardinalityIssue::IncompleteProvider),
        ("neMissingRight", CardinalityIssue::IncompleteProvider),
        ("neMissingLeft", CardinalityIssue::IncompleteProvider),
        ("sameMissingRight", CardinalityIssue::IncompleteProvider),
        ("sameMissingLeft", CardinalityIssue::IncompleteProvider),
        (
            "differentMissingRight",
            CardinalityIssue::IncompleteProvider,
        ),
        ("differentMissingLeft", CardinalityIssue::IncompleteProvider),
        ("divisionRight", CardinalityIssue::UnsupportedBound),
        ("divisionLeft", CardinalityIssue::UnsupportedBound),
        ("overflow", CardinalityIssue::UnsupportedBound),
        ("real", CardinalityIssue::UnsupportedBound),
        ("infinity", CardinalityIssue::UnsupportedBound),
        ("sequence", CardinalityIssue::UnsupportedBound),
        ("recursive", CardinalityIssue::UnsupportedBound),
        ("booleanEndpoint", CardinalityIssue::UnsupportedBound),
        ("emptySelected", CardinalityIssue::UnsupportedBound),
        ("numericCondition", CardinalityIssue::UnsupportedBound),
        ("eagerMissing", CardinalityIssue::IncompleteProvider),
    ] {
        let e = q.find(&format!("EmptyComparisonGuards::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Unknown { reason },
            "{name}"
        );
    }
}

fn string_snapshot() -> &'static Snapshot {
    static VALUE: OnceLock<Snapshot> = OnceLock::new();
    VALUE.get_or_init(|| {
        let original = analyze(vec![
            document(
                "strings.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-033.sysml"),
            ),
            document(
                "guards.sysml",
                include_str!("../../../tests/conformance/fixtures/mult-034.sysml"),
            ),
        ]);
        assert!(!original
            .diagnostics
            .iter()
            .any(|d| matches!(d.stage, Stage::Syntax)));
        serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap()
    })
}
#[test]
fn strings_preserve_decoded_values_and_presence_after_round_trip() {
    let q = string_snapshot().queries().unwrap();
    for (name, value) in [
        ("eqEqual", "7"),
        ("eqCase", "9"),
        ("eqEmpty", "7"),
        ("neEqual", "9"),
        ("neCase", "7"),
        ("neEmpty", "9"),
        ("sameEqual", "7"),
        ("sameCase", "9"),
        ("sameEmpty", "7"),
        ("differentEqual", "9"),
        ("differentCase", "7"),
        ("differentEmpty", "9"),
        ("unicode", "7"),
        ("distinctUnicode", "7"),
        ("escapedBackslash", "7"),
        ("escapedQuote", "7"),
        ("allEscapes", "7"),
        ("concatenate", "7"),
        ("unicodeConcatenate", "7"),
        ("nestedConcatenate", "7"),
        ("emptyConcatenate", "7"),
        ("viaAlias", "7"),
        ("nullRight", "9"),
        ("nullLeft", "7"),
        ("emptyStringPresent", "7"),
        ("nullFallback", "7"),
        ("skipMissing", "7"),
        ("selectedString", "7"),
        ("logicalSkip", "7"),
    ] {
        let e = q.find(&format!("StringValues::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Evaluated {
                lower: value.into(),
                upper: Some(value.into())
            },
            "{name}"
        );
    }
}
#[test]
fn strings_preserve_unknown_operands_and_unsupported_domains() {
    let q = string_snapshot().queries().unwrap();
    for (name, reason) in [
        ("stringEndpoint", CardinalityIssue::UnsupportedBound),
        ("concatEndpoint", CardinalityIssue::UnsupportedBound),
        ("stringCondition", CardinalityIssue::UnsupportedBound),
        ("unaryPlus", CardinalityIssue::UnsupportedBound),
        ("unaryMinus", CardinalityIssue::UnsupportedBound),
        ("subtract", CardinalityIssue::UnsupportedBound),
        ("numberConcat", CardinalityIssue::UnsupportedBound),
        ("boolConcat", CardinalityIssue::UnsupportedBound),
        ("nullConcat", CardinalityIssue::UnsupportedBound),
        ("sequenceConcat", CardinalityIssue::UnsupportedBound),
        ("numberEquality", CardinalityIssue::UnsupportedBound),
        ("boolEquality", CardinalityIssue::UnsupportedBound),
        ("missingRight", CardinalityIssue::IncompleteProvider),
        ("missingLeft", CardinalityIssue::IncompleteProvider),
        ("selectedMissing", CardinalityIssue::IncompleteProvider),
        ("recursive", CardinalityIssue::UnsupportedBound),
        ("less", CardinalityIssue::UnsupportedBound),
        ("lessEqual", CardinalityIssue::UnsupportedBound),
        ("greater", CardinalityIssue::UnsupportedBound),
        ("greaterEqual", CardinalityIssue::UnsupportedBound),
    ] {
        let e = q.find(&format!("StringGuards::{name}"))[0];
        assert_eq!(
            q.cardinality(&e.id).unwrap(),
            &Cardinality::Unknown { reason },
            "{name}"
        );
    }
}

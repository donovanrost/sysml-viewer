use super::{Antecedent, antecedent};
use crate::{
    json::{
        ClosurePolicy, ElementRef, FeatureTypeIssue, ResolvedModel,
        structural_index::StoredStructure,
        type_relations::{RelationFact, TypeRelations},
    },
    model::{GraphFormat, Model},
    prepared::PreparedLibrary,
};
use serde_json::json;
use std::sync::{Arc, OnceLock};
fn raw_library() -> Model {
    let mut m = Model::with_graph_format(GraphFormat::CanonicalV3);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec-refs/SysML-v2-Release/sysml.library");
    assert_eq!(m.load_library_dir(&path).unwrap(), 94);
    m
}
fn library() -> Arc<PreparedLibrary> {
    static LIBRARY: OnceLock<Arc<PreparedLibrary>> = OnceLock::new();
    Arc::clone(LIBRARY.get_or_init(|| raw_library().prepare_library().unwrap()))
}
fn resolved(mut m: Model, source: &str) -> ResolvedModel {
    let parsed = m.add_source("portion-acceptance.kerml", source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    ResolvedModel::build(&m)
}
fn fixture(source: &str) -> ResolvedModel {
    let mut m = Model::with_graph_format(GraphFormat::CanonicalV3);
    library().install(&mut m).unwrap();
    resolved(m, source)
}
fn ready(r: &mut ResolvedModel) {
    r.ensure_implied();
    r.set_closure_policy(ClosurePolicy::Closure {
        include_implied: true,
    });
}
fn exact(r: &mut ResolvedModel, name: &str, ty: &str) -> uuid::Uuid {
    let f = r.resolve_qualified(name).unwrap();
    let t = r.resolve_qualified(ty).unwrap();
    let rows = r.b.elements.len();
    assert_eq!(r.feature_type_report(f).types, Ok(vec![t]), "{name}");
    assert_eq!(
        r.property(f, "type").unwrap(),
        json!([{"@id":r.element_id(t)}]),
        "{name}"
    );
    assert_eq!(r.b.elements.len(), rows);
    r.element_id(f)
}
const SOURCE: &str = r#"
class Owner {feature plain:Occurrences::Occurrence;portion feature direct:Occurrences::Occurrence;portion feature explicit:Occurrences::Occurrence subsets Occurrences::Occurrence::portions;portion feature data:ScalarValues::Integer;}
feature owner:Owner {portion feature nested:Occurrences::Occurrence;}
feature seed:Owner;
feature inherited subsets seed {portion feature nested:Occurrences::Occurrence;}
classifier Narrow specializes Owner;
feature narrowed:Owner,Narrow {portion feature nested:Occurrences::Occurrence;}
portion feature outside:Occurrences::Occurrence;
feature anyOwner:Base::Anything {portion feature child:Occurrences::Occurrence;}
feature cyclic:Owner subsets cyclicPeer {portion feature nested:Occurrences::Occurrence;}
feature cyclicPeer subsets cyclic;
"#;
#[test]
fn portion_types_and_antecedents_use_owned_typing_and_complete_effective_owner_types() {
    let mut r = fixture(SOURCE);
    ready(&mut r);
    for (name, ty) in [
        ("Owner::direct", "Occurrences::Occurrence"),
        ("Owner::explicit", "Occurrences::Occurrence"),
        ("Owner::data", "ScalarValues::Integer"),
        ("owner::nested", "Occurrences::Occurrence"),
        ("inherited::nested", "Occurrences::Occurrence"),
        ("narrowed::nested", "Occurrences::Occurrence"),
        ("outside", "Occurrences::Occurrence"),
        ("anyOwner::child", "Occurrences::Occurrence"),
        ("cyclic::nested", "Occurrences::Occurrence"),
        (
            "Occurrences::Occurrence::portions",
            "Occurrences::Occurrence",
        ),
        (
            "Occurrences::Occurrence::timeSlices",
            "Occurrences::Occurrence",
        ),
        (
            "Occurrences::Occurrence::spaceSlices",
            "Occurrences::Occurrence",
        ),
        ("Occurrences::Occurrence::self", "Occurrences::Occurrence"),
        (
            "Performances::Performance::self",
            "Performances::Performance",
        ),
    ] {
        exact(&mut r, name, ty);
    }
    let raw = StoredStructure::for_query(&mut r.b, &mut 0).unwrap();
    let mut relations = TypeRelations::with_structure(Arc::clone(&raw));
    for name in ["Owner::plain", "Owner::data", "outside"] {
        let f = r.resolve_qualified(name).unwrap();
        assert_eq!(
            antecedent(&mut r.b, f.0, &raw, &mut relations, &mut 0),
            Some(Antecedent::Inapplicable),
            "{name}"
        );
    }
    let direct = r.resolve_qualified("Owner::direct").unwrap();
    assert_eq!(
        antecedent(&mut r.b, direct.0, &raw, &mut relations, &mut 0),
        Some(Antecedent::Applies)
    );
    let owners: Vec<_> = ["owner", "inherited", "narrowed", "anyOwner", "cyclic"]
        .into_iter()
        .map(|name| r.resolve_qualified(name).unwrap().0)
        .collect();
    let values = crate::json::feature_type_projection::owners_of_kind_for_planning(
        &mut r.b,
        &owners,
        Arc::new(Default::default()),
        "Class",
        &mut 0,
    )
    .unwrap();
    for (owner, expected) in owners.into_iter().zip([true, true, false, false, true]) {
        assert_eq!(values.get(&owner), Some(&expected));
    }
    let unrelated = r.resolve_qualified("ScalarValues::Boolean").unwrap();
    assert_eq!(
        TypeRelations::default().specializes(&mut r.b, direct.0, unrelated.0, &mut 0),
        RelationFact::No
    );
}
#[derive(Clone, Copy, Debug)]
enum Damage {
    Flag,
    TypingMissing,
    TypingSource,
    TypingCarrier,
    DuplicateCarrier,
    TargetAlias,
    OrphanTyping,
    DuplicateOwned,
    OwnerCarrier,
    LibraryName,
}
const DAMAGES: [Damage; 10] = [
    Damage::Flag,
    Damage::TypingMissing,
    Damage::TypingSource,
    Damage::TypingCarrier,
    Damage::DuplicateCarrier,
    Damage::TargetAlias,
    Damage::OrphanTyping,
    Damage::DuplicateOwned,
    Damage::OwnerCarrier,
    Damage::LibraryName,
];
fn damage(r: &mut ResolvedModel, f: ElementRef, change: Damage) {
    let rel = r
        .owned_relationships(f)
        .into_iter()
        .find(|&e| r.element_type(e) == "FeatureTyping")
        .unwrap()
        .0;
    let other = r.resolve_qualified("Owner::plain").unwrap();
    let id = r.element_id(other);
    match change {
        Damage::Flag => r.b.set(f.0, "isPortion", json!("true")),
        Damage::TypingMissing => r.b.set(
            rel,
            "type",
            json!({"@id":"00000000-0000-0000-0000-000000000000"}),
        ),
        Damage::TypingSource => r.b.set(rel, "typedFeature", json!({"@id":id})),
        Damage::TypingCarrier => r.b.set(rel, "owningRelatedElement", json!({"@id":id})),
        Damage::DuplicateCarrier => r.b.elements[other.0].owned_relationships.push(rel),
        Damage::TargetAlias => r.b.set(rel, "target", json!([{"@id":id}])),
        Damage::OrphanTyping => r.b.elements[f.0]
            .owned_relationships
            .make_mut()
            .retain(|&e| e != rel),
        Damage::DuplicateOwned => r.b.elements[f.0].owned_relationships.push(rel),
        Damage::OwnerCarrier => {
            let membership = r.b.elements[f.0].owning_relationship.unwrap();
            r.b.set(membership, "owningRelatedElement", json!({"@id":id}));
        }
        Damage::LibraryName => {
            let t = r
                .resolve_qualified("Occurrences::Occurrence::portions")
                .unwrap();
            r.b.set(t.0, "declaredName", json!("renamedPortions"));
        }
    }
}
#[test]
fn malformed_portion_evidence_cannot_certify_types_or_negative_specialization() {
    for change in DAMAGES {
        let mut r = fixture(SOURCE);
        let f = r.resolve_qualified("Owner::direct").unwrap();
        damage(&mut r, f, change);
        ready(&mut r);
        let report = r.feature_type_report(f);
        assert!(report.types.is_err(), "{change:?}: {report:?}");
        assert_ne!(report.types, Err(FeatureTypeIssue::WorkLimit), "{change:?}");
        let unrelated = r.resolve_qualified("ScalarValues::Boolean").unwrap();
        let mut steps = 0;
        assert_eq!(
            TypeRelations::default().specializes(&mut r.b, f.0, unrelated.0, &mut steps),
            RelationFact::Unknown,
            "{change:?}"
        );
        assert!(steps <= crate::eval::MAX_STEPS);
    }
}
#[test]
fn portion_edits_revoke_previous_complete_type_results() {
    for change in DAMAGES {
        let mut r = fixture(SOURCE);
        ready(&mut r);
        exact(&mut r, "Owner::direct", "Occurrences::Occurrence");
        let f = r.resolve_qualified("Owner::direct").unwrap();
        damage(&mut r, f, change);
        assert!(r.feature_type_report(f).types.is_err(), "{change:?}");
    }
}
#[test]
fn ordinary_intersections_require_every_exact_subsetting_witness() {
    for (subsets, complete) in [("a,b", true), ("a", false)] {
        let source = format!(
            "feature a:Occurrences::Occurrence;feature b:Occurrences::Occurrence;feature both:Occurrences::Occurrence subsets {subsets} intersects a,b;"
        );
        let mut r = fixture(&source);
        ready(&mut r);
        if complete {
            exact(&mut r, "both", "Occurrences::Occurrence");
        } else {
            let f = r.resolve_qualified("both").unwrap();
            let report = r.feature_type_report(f);
            assert!(report.types.is_err());
            assert_ne!(report.types, Err(FeatureTypeIssue::WorkLimit));
        }
    }
}
#[test]
fn portion_types_preserve_raw_prepared_serialized_identity_and_budget_retry() {
    let prepared = library();
    let decoded =
        Arc::new(PreparedLibrary::from_bytes(&prepared.to_bytes(153).unwrap(), 153).unwrap());
    let mut ids = None;
    for replay in [None, Some(prepared), Some(decoded)] {
        let model = if let Some(replay) = replay {
            let mut m = Model::with_graph_format(GraphFormat::CanonicalV3);
            replay.install(&mut m).unwrap();
            m
        } else {
            raw_library()
        };
        let mut r = resolved(model, SOURCE);
        ready(&mut r);
        let actual = [
            exact(&mut r, "Owner::direct", "Occurrences::Occurrence"),
            exact(&mut r, "inherited::nested", "Occurrences::Occurrence"),
            exact(&mut r, "cyclic::nested", "Occurrences::Occurrence"),
            exact(
                &mut r,
                "Occurrences::Occurrence::self",
                "Occurrences::Occurrence",
            ),
        ];
        if let Some(ids) = ids {
            assert_eq!(actual, ids)
        } else {
            ids = Some(actual)
        }
        let f = r.resolve_qualified("Owner::direct").unwrap();
        assert_eq!(
            r.feature_type_report_with_budget(f, crate::eval::MAX_STEPS)
                .types,
            Err(FeatureTypeIssue::WorkLimit)
        );
        let t = r.resolve_qualified("Occurrences::Occurrence").unwrap();
        let rows = r.b.elements.len();
        let retry = r.feature_type_report_with_budget(f, 100);
        assert_eq!(retry.types, Ok(vec![t]));
        assert!(retry.steps > 100);
        assert_eq!(r.b.elements.len(), rows);
    }
}

#[test]
fn feature_owner_portions_have_complete_negative_specialization_proofs() {
    let mut r = fixture(SOURCE);
    ready(&mut r);
    let unrelated = r.resolve_qualified("ScalarValues::Boolean").unwrap();
    let portions = r
        .resolve_qualified("Occurrences::Occurrence::portions")
        .unwrap();
    let mut failures = Vec::new();
    for name in [
        "owner::nested",
        "inherited::nested",
        "narrowed::nested",
        "anyOwner::child",
        "cyclic::nested",
    ] {
        let f = r.resolve_qualified(name).unwrap();
        let expected_path = if matches!(name, "narrowed::nested" | "anyOwner::child") {
            RelationFact::No
        } else {
            RelationFact::Yes
        };
        assert_eq!(
            TypeRelations::default().specializes(&mut r.b, f.0, portions.0, &mut 0),
            expected_path,
            "{name} conditional portion path"
        );
        let mut steps = 0;
        let fact = TypeRelations::default().specializes(&mut r.b, f.0, unrelated.0, &mut steps);
        assert!(steps <= crate::eval::MAX_STEPS);
        if fact != RelationFact::No {
            failures.push((name, fact, steps));
        }
    }
    assert!(failures.is_empty(), "Incomplete negatives: {failures:?}");
}

#[test]
fn selected_default_results_expose_exact_types_without_coercion_to_valued_features() {
    let mut r = fixture(
        "function F {in x:ScalarValues::Integer default=1;return result:ScalarValues::Integer;} feature call=F();",
    );
    let call = r
        .user_elements()
        .find(|&e| r.element_type(e) == "InvocationExpression")
        .unwrap();
    ready(&mut r);
    let defaults = r
        .invocation_default_report(call)
        .selection
        .unwrap()
        .defaults;
    let expected = [
        ("F::x", "ScalarValues::Integer"),
        (
            "Performances::Performance::isDispatch",
            "ScalarValues::Boolean",
        ),
        (
            "Performances::Performance::dispatchScope",
            "Performances::Performance",
        ),
        (
            "Performances::Performance::thisPerformance",
            "Performances::Performance",
        ),
        (
            "Occurrences::Occurrence::portionOfLife",
            "Occurrences::Occurrence",
        ),
        ("Occurrences::Occurrence::this", "Occurrences::Occurrence"),
        (
            "Occurrences::Occurrence::localClock",
            "Clocks::UniversalClockLife",
        ),
        (
            "Occurrences::Occurrence::isRunToCompletion",
            "ScalarValues::Boolean",
        ),
        (
            "Occurrences::Occurrence::runToCompletionScope",
            "Occurrences::Occurrence",
        ),
        (
            "Occurrences::Occurrence::incomingTransferSort",
            "Occurrences::IncomingTransferSort",
        ),
    ];
    assert_eq!(defaults.len(), expected.len());
    let rows = r.b.elements.len();
    for (default, (name, ty)) in defaults.into_iter().zip(expected) {
        assert_eq!(
            r.element_qualified_name(default.feature).as_deref(),
            Some(name)
        );
        let result = r
            .expression_result_report(default.value)
            .result
            .unwrap()
            .unwrap();
        let expected_type = r.resolve_qualified(ty).unwrap();
        assert_eq!(
            r.feature_type_report(result).types,
            Ok(vec![expected_type]),
            "{name}"
        );
        assert_eq!(
            r.property(result, "type").unwrap(),
            json!([{"@id":r.element_id(expected_type)}]),
            "{name}"
        );
    }
    assert_eq!(r.b.elements.len(), rows);
    // The default is self (Occurrence). The valued Feature itself is typed Life;
    // validating that binding is a separate obligation, not a result type cast.
    exact(
        &mut r,
        "Occurrences::Occurrence::portionOfLife",
        "Occurrences::Life",
    );
}

#[test]
fn nonowned_and_inherited_typing_do_not_activate_the_owned_class_antecedent() {
    let mut r = fixture(
        r#"
        class Owner {
            portion feature bareStandalone;
            portion feature standalone subsets Occurrences::occurrences;
            portion feature inherited subsets source;
            feature source:Occurrences::Occurrence;
            portion feature owned:Occurrences::Occurrence;
        }
        typing Owner::standalone typed by Occurrences::Occurrence;
        typing Owner::bareStandalone typed by Occurrences::Occurrence;
    "#,
    );
    ready(&mut r);
    let raw = StoredStructure::for_query(&mut r.b, &mut 0).unwrap();
    let mut relations = TypeRelations::with_structure(Arc::clone(&raw));
    let bare = r.resolve_qualified("Owner::bareStandalone").unwrap();
    assert_eq!(
        antecedent(&mut r.b, bare.0, &raw, &mut relations, &mut 0),
        Some(Antecedent::Inapplicable)
    );
    exact(&mut r, "Owner::bareStandalone", "Occurrences::Occurrence");
    let unrelated = r.resolve_qualified("ScalarValues::Boolean").unwrap();
    assert_eq!(
        TypeRelations::default().specializes(&mut r.b, bare.0, unrelated.0, &mut 0),
        RelationFact::No
    );
    for (name, applies) in [
        ("Owner::standalone", false),
        ("Owner::inherited", false),
        ("Owner::owned", true),
    ] {
        let f = r.resolve_qualified(name).unwrap();
        assert_eq!(
            antecedent(&mut r.b, f.0, &raw, &mut relations, &mut 0),
            Some(if applies {
                Antecedent::Applies
            } else {
                Antecedent::Inapplicable
            }),
            "{name}"
        );
        exact(&mut r, name, "Occurrences::Occurrence");
    }
}

#[test]
fn owner_projection_negative_proofs_exclude_future_writers_and_retry_budgets() {
    let mut r = fixture(SOURCE);
    ready(&mut r);
    let unrelated = r.resolve_qualified("ScalarValues::Boolean").unwrap();
    let raw = StoredStructure::for_query(&mut r.b, &mut 0).unwrap();
    for (name, writer) in [("owner::nested", "owner"), ("inherited::nested", "seed")] {
        let f = r.resolve_qualified(name).unwrap();
        let writer = r.resolve_qualified(writer).unwrap();
        let mut relations = TypeRelations::with_stable_specializations(
            Arc::clone(&raw),
            Arc::new([writer.0].into_iter().collect()),
        );
        let mut steps = 0;
        assert_eq!(
            relations.specializes(&mut r.b, f.0, unrelated.0, &mut steps),
            RelationFact::Unknown,
            "{name}"
        );
        assert!(steps <= crate::eval::MAX_STEPS);
        assert_eq!(
            TypeRelations::default().specializes(&mut r.b, f.0, unrelated.0, &mut 0),
            RelationFact::No,
            "{name}"
        );
    }
    let f = r.resolve_qualified("owner::nested").unwrap();
    let mut relations = TypeRelations::default();
    let mut steps = crate::eval::MAX_STEPS;
    assert_eq!(
        relations.specializes(&mut r.b, f.0, unrelated.0, &mut steps),
        RelationFact::Unknown
    );
    assert!(steps > crate::eval::MAX_STEPS);
    relations.reset_with_budget(&mut 0).unwrap();
    assert_eq!(
        relations.specializes(&mut r.b, f.0, unrelated.0, &mut 0),
        RelationFact::No
    );
}

#[test]
fn damaged_owner_types_revoke_negatives_and_coupled_contexts_remain_unknown() {
    let mut r = fixture(SOURCE);
    ready(&mut r);
    let unrelated = r.resolve_qualified("ScalarValues::Boolean").unwrap();
    let f = r.resolve_qualified("owner::nested").unwrap();
    let owner = r.resolve_qualified("owner").unwrap();
    let mut relations = TypeRelations::default();
    assert_eq!(
        relations.specializes(&mut r.b, f.0, unrelated.0, &mut 0),
        RelationFact::No
    );
    let typing = r
        .owned_relationships(owner)
        .into_iter()
        .find(|&e| r.element_type(e) == "FeatureTyping")
        .unwrap();
    r.b.set(
        typing.0,
        "type",
        json!({"@id":"00000000-0000-0000-0000-000000000000"}),
    );
    assert_eq!(
        relations.specializes(&mut r.b, f.0, unrelated.0, &mut 0),
        RelationFact::Unknown
    );
    assert_eq!(
        TypeRelations::default().specializes(&mut r.b, f.0, unrelated.0, &mut 0),
        RelationFact::Unknown
    );
    assert!(r.feature_type_report(f).types.is_err());

    let mut r = fixture(&format!(
        "{SOURCE} feature coupled:Owner subsets coupled::nested {{portion feature nested:Occurrences::Occurrence;}}"
    ));
    ready(&mut r);
    let f = r.resolve_qualified("coupled::nested").unwrap();
    let unrelated = r.resolve_qualified("ScalarValues::Boolean").unwrap();
    let mut relations = TypeRelations::default();
    let mut steps = 0;
    assert_eq!(
        relations.specializes(&mut r.b, f.0, unrelated.0, &mut steps),
        RelationFact::Unknown
    );
    assert!(steps <= crate::eval::MAX_STEPS);
    let healthy = r.resolve_qualified("owner::nested").unwrap();
    assert_eq!(
        relations.specializes(&mut r.b, healthy.0, unrelated.0, &mut 0),
        RelationFact::No
    );
}

#[test]
fn standalone_typing_does_not_add_owned_typing_family_obligations() {
    let mut r = fixture(
        r#"
        feature occurrence; typing occurrence typed by Occurrences::Occurrence;
        feature object; typing object typed by Objects::Object;
        feature data; typing data typed by ScalarValues::Integer;
        feature ownedOccurrence:Occurrences::Occurrence;
        feature ownedObject:Objects::Object;
        feature ownedData:ScalarValues::Integer;
    "#,
    );
    ready(&mut r);
    for (name, owned, ty, base) in [
        (
            "occurrence",
            "ownedOccurrence",
            "Occurrences::Occurrence",
            "Occurrences::occurrences",
        ),
        (
            "object",
            "ownedObject",
            "Objects::Object",
            "Objects::objects",
        ),
        (
            "data",
            "ownedData",
            "ScalarValues::Integer",
            "Base::dataValues",
        ),
    ] {
        exact(&mut r, name, ty);
        exact(&mut r, owned, ty);
        let f = r.resolve_qualified(name).unwrap();
        let owned = r.resolve_qualified(owned).unwrap();
        let base = r.resolve_qualified(base).unwrap();
        assert_eq!(
            TypeRelations::default().specializes(&mut r.b, f.0, base.0, &mut 0),
            RelationFact::No,
            "{name}"
        );
        assert_eq!(
            TypeRelations::default().specializes(&mut r.b, owned.0, base.0, &mut 0),
            RelationFact::Yes,
            "{name}"
        );
    }
}

#[test]
fn standalone_typing_negative_proofs_reject_conflicting_sources_endpoints_and_carriers() {
    for change in 0..8 {
        let mut r = fixture(
            "feature f; typing f typed by ScalarValues::Integer; feature other:ScalarValues::String;",
        );
        ready(&mut r);
        let f = r.resolve_qualified("f").unwrap();
        let other = r.resolve_qualified("other").unwrap();
        let unrelated = r.resolve_qualified("ScalarValues::Boolean").unwrap();
        let row = r
            .user_elements()
            .find(|&e| {
                r.element_type(e) == "FeatureTyping"
                    && r.b.elements[e.0]
                        .props
                        .get("typedFeature")
                        .and_then(|v| v.as_reference())
                        == Some(r.element_id(f))
            })
            .unwrap();
        let mut held = TypeRelations::default();
        assert_eq!(
            held.specializes(&mut r.b, f.0, unrelated.0, &mut 0),
            RelationFact::No
        );
        let other_id = r.element_id(other);
        match change {
            0 => r.b.set(row.0, "specific", json!({"@id":other_id})),
            1 => r.b.set(row.0, "source", json!([{"@id":other_id}])),
            2 => r.b.set(row.0, "target", json!([{"@id":other_id}])),
            3 => r.b.set(
                row.0,
                "type",
                json!({"@id":"00000000-0000-0000-0000-000000000000"}),
            ),
            4 => {
                r.b.set(row.0, "owningRelatedElement", json!({"@id":other_id}))
            }
            5 => r.b.elements[other.0].owned_relationships.push(row.0),
            6 => {
                r.b.elements[other.0].owned_relationships.push(row.0);
                r.b.elements[f.0].owned_relationships.push(row.0);
            }
            _ => {
                r.b.set(row.0, "typedFeature", json!(null));
                r.b.set(row.0, "specific", json!(null));
            }
        }
        assert_eq!(
            held.specializes(&mut r.b, f.0, unrelated.0, &mut 0),
            RelationFact::Unknown,
            "held {change}"
        );
        let mut steps = 0;
        assert_eq!(
            TypeRelations::default().specializes(&mut r.b, f.0, unrelated.0, &mut steps),
            RelationFact::Unknown,
            "fresh {change}"
        );
        assert!(
            steps <= crate::eval::MAX_STEPS,
            "change {change}: {steps} steps"
        );
    }
}

#[test]
fn portion_graph_formats_keep_legacy_missing_owner_bases_explicit() {
    for format in [GraphFormat::LegacyV2, GraphFormat::CanonicalV3] {
        let mut m = Model::with_graph_format(format);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../spec-refs/SysML-v2-Release/sysml.library");
        assert_eq!(m.load_library_dir(&path).unwrap(), 94);
        let mut r = resolved(
            m,
            r#"
            class Owner {portion feature direct:Occurrences::Occurrence;}
            feature owner:Owner {portion feature nested:Occurrences::Occurrence;}
            feature explicitOwner:Owner subsets Occurrences::occurrences {
                portion feature nested:Occurrences::Occurrence subsets Occurrences::Occurrence::portions;
            }
        "#,
        );
        ready(&mut r);
        let unrelated = r.resolve_qualified("ScalarValues::Boolean").unwrap();
        for name in [
            "Owner::direct",
            "owner::nested",
            "explicitOwner::nested",
            "Occurrences::Occurrence::portions",
        ] {
            let f = r.resolve_qualified(name).unwrap();
            let negative = TypeRelations::default().specializes(&mut r.b, f.0, unrelated.0, &mut 0);
            if format == GraphFormat::LegacyV2 && name == "owner::nested" {
                assert_eq!(
                    r.feature_type_report(f).types,
                    Err(FeatureTypeIssue::MissingRequiredFamilies)
                );
                assert_eq!(negative, RelationFact::Unknown);
            } else {
                exact(&mut r, name, "Occurrences::Occurrence");
                assert_eq!(negative, RelationFact::No, "{format:?} {name}");
            }
        }
    }
}

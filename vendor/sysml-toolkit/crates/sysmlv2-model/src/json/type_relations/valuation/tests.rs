use crate::{
    json::{ElementRef, FeatureTypeIssue, ResolvedModel},
    model::{GraphFormat, Model},
    prepared::PreparedLibrary,
};
use serde_json::json;
use std::sync::{Arc, OnceLock};

fn raw_library() -> Model {
    let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec-refs/SysML-v2-Release/sysml.library");
    assert_eq!(model.load_library_dir(&root).unwrap(), 94);
    model
}
fn library() -> Arc<PreparedLibrary> {
    static LIBRARY: OnceLock<Arc<PreparedLibrary>> = OnceLock::new();
    Arc::clone(LIBRARY.get_or_init(|| raw_library().prepare_library().unwrap()))
}

fn fixture(source: &str) -> ResolvedModel {
    let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
    library().install(&mut model).unwrap();
    assert!(
        model
            .add_source("applicability.kerml", source)
            .diagnostics
            .is_empty()
    );
    ResolvedModel::build(&model)
}

fn integer(r: &mut ResolvedModel, name: &str) -> uuid::Uuid {
    let feature = r.resolve_qualified(name).unwrap();
    let expected = r.resolve_qualified("ScalarValues::Integer").unwrap();
    let rows = r.b.elements.len();
    let report = r.feature_type_report(feature);
    assert_eq!(report.types, Ok(vec![expected]), "{name}: {report:?}");
    assert_eq!(r.b.elements.len(), rows);
    r.element_id(feature)
}

#[test]
fn static_types_do_not_depend_on_evaluating_or_accepting_the_value() {
    let mut r = fixture(
        r#"
        feature mismatch:ScalarValues::Integer = "text";
        feature unresolved:ScalarValues::Integer default = missing;
        feature left:ScalarValues::Integer = right;
        feature right:ScalarValues::Integer = left;
        function F {in x:ScalarValues::Integer default = missing; return r;}
    "#,
    );
    r.ensure_implied();
    for name in ["mismatch", "unresolved", "left", "right", "F::x"] {
        integer(&mut r, name);
    }
}

#[derive(Debug, Clone, Copy)]
enum Damage {
    MissingFeatureOwner,
    MissingValueOwner,
    ForeignValueCarrier,
    DefaultFlag,
    InitialFlag,
    WrongValueKind,
    MalformedDirection,
    ImpliedTyping,
    MalformedImplied,
    ForeignTypingSource,
    MissingTypingTarget,
    Metadata,
}
const DAMAGES: [Damage; 12] = [
    Damage::MissingFeatureOwner,
    Damage::MissingValueOwner,
    Damage::ForeignValueCarrier,
    Damage::DefaultFlag,
    Damage::InitialFlag,
    Damage::WrongValueKind,
    Damage::MalformedDirection,
    Damage::ImpliedTyping,
    Damage::MalformedImplied,
    Damage::ForeignTypingSource,
    Damage::MissingTypingTarget,
    Damage::Metadata,
];
fn damage(r: &mut ResolvedModel, feature: ElementRef, change: Damage) {
    let relations = r.owned_relationships(feature);
    let valuation = *relations
        .iter()
        .find(|&&e| r.element_type(e) == "FeatureValue")
        .unwrap();
    let typing = *relations
        .iter()
        .find(|&&e| r.element_type(e) == "FeatureTyping")
        .unwrap();
    let value = r.membership_member(valuation).unwrap();
    let control = r.resolve_qualified("control").unwrap();
    let wrong = r.element_id(control);
    match change {
        Damage::MissingFeatureOwner => r.b.elements[feature.0].owning_relationship = None,
        Damage::MissingValueOwner => r.b.elements[value.0].owning_relationship = None,
        Damage::ForeignValueCarrier => {
            r.b.set(valuation.0, "owningRelatedElement", json!({"@id":wrong}))
        }
        Damage::DefaultFlag => r.b.set(valuation.0, "isDefault", json!("invalid")),
        Damage::InitialFlag => r.b.set(valuation.0, "isInitial", json!("invalid")),
        Damage::WrongValueKind => r.b.elements[value.0].ty = "Feature",
        Damage::MalformedDirection => r.b.set(feature.0, "direction", json!("invalid")),
        Damage::ImpliedTyping => r.b.set(typing.0, "isImplied", json!(true)),
        Damage::MalformedImplied => r.b.set(typing.0, "isImplied", json!("invalid")),
        Damage::ForeignTypingSource => r.b.set(typing.0, "typedFeature", json!({"@id":wrong})),
        Damage::MissingTypingTarget => r.b.set(
            typing.0,
            "type",
            json!({"@id":"00000000-0000-0000-0000-000000000000"}),
        ),
        Damage::Metadata => r.b.metadata_associations_incomplete = true,
    }
}
const SOURCE: &str =
    "feature control:ScalarValues::Integer; feature selected:ScalarValues::Integer default=1;";

#[test]
fn false_antecedent_requires_valid_membership_flags_and_specialization_evidence() {
    for change in DAMAGES {
        let mut r = fixture(SOURCE);
        let feature = r.resolve_qualified("selected").unwrap();
        damage(&mut r, feature, change);
        r.ensure_implied();
        let report = r.feature_type_report(feature);
        assert!(report.types.is_err(), "{change:?}: {report:?}");
        assert_ne!(report.types, Err(FeatureTypeIssue::WorkLimit), "{change:?}");
    }
}

#[test]
fn type_projection_does_not_reuse_success_after_valuation_edits() {
    for change in DAMAGES {
        let mut r = fixture(SOURCE);
        r.ensure_implied();
        integer(&mut r, "selected");
        let feature = r.resolve_qualified("selected").unwrap();
        damage(&mut r, feature, change);
        assert!(r.feature_type_report(feature).types.is_err(), "{change:?}");
    }
}

#[test]
fn valued_static_types_preserve_identities_through_raw_prepared_and_serialized_replay() {
    let prepared = library();
    let decoded =
        Arc::new(PreparedLibrary::from_bytes(&prepared.to_bytes(146).unwrap(), 146).unwrap());
    let mut expected = None;
    for library in [None, Some(prepared), Some(decoded)] {
        let mut model = if let Some(library) = library {
            let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
            library.install(&mut model).unwrap();
            model
        } else {
            raw_library()
        };
        assert!(
            model
                .add_source("applicability.kerml", SOURCE)
                .diagnostics
                .is_empty()
        );
        let mut r = ResolvedModel::build(&model);
        r.ensure_implied();
        let id = integer(&mut r, "selected");
        if let Some(expected) = expected {
            assert_eq!(id, expected);
        } else {
            expected = Some(id);
        }
    }
}

#[test]
fn directed_and_explicit_specializations_preserve_declared_types() {
    let mut r = fixture(
        r#"
function F {
  in control : ScalarValues::Integer;
  in inputDefault : ScalarValues::Integer default = 1;
  out outputDefault : ScalarValues::Integer default = 1;
  inout inoutDefault : ScalarValues::Integer default = 1;
  in inputValue : ScalarValues::Integer = 1;
  return result : ScalarValues::Integer default = 1;
}
datatype D specializes ScalarValues::Integer;
feature typedControl : D;
feature typedDefault : D default = 1;
feature typedValue : D = 1;
feature subsetControl subsets typedControl;
feature subsetDefault subsets typedControl default = 1;
feature subsetValue subsets typedControl = 1;
feature inferredDefault default = 1;
feature inferredValue = 1;
class A {feature original : D;}
class B specializes A {
  feature renamed redefines A::original default = 1;
}
function Parent {in x : D; return r : D;}
function Child specializes Parent {in x redefines Parent::x default = 1;}
"#,
    );
    r.ensure_implied();
    for name in [
        "F::control",
        "F::inputDefault",
        "F::outputDefault",
        "F::inoutDefault",
        "F::inputValue",
        "F::result",
        "typedControl",
        "typedDefault",
        "typedValue",
        "subsetControl",
        "subsetDefault",
        "subsetValue",
        "inferredDefault",
        "inferredValue",
        "B::renamed",
        "Parent::x",
        "Child::x",
    ] {
        let feature = r.resolve_qualified(name).unwrap();
        let report = r.feature_type_report(feature);
        if matches!(name, "inferredDefault" | "inferredValue") {
            assert!(report.types.is_err(), "{name}");
            assert_ne!(report.types, Err(FeatureTypeIssue::WorkLimit), "{name}");
        } else {
            let expected = r
                .resolve_qualified(if name.starts_with("F::") {
                    "ScalarValues::Integer"
                } else {
                    "D"
                })
                .unwrap();
            assert_eq!(report.types, Ok(vec![expected]), "{name}");
        }
    }
}

#[test]
fn valuation_queries_preserve_cumulative_budget_and_retry_after_exhaustion() {
    let mut r = fixture(SOURCE);
    r.ensure_implied();
    let feature = r.resolve_qualified("selected").unwrap();
    let rows = r.b.elements.len();
    assert_eq!(
        r.feature_type_report_with_budget(feature, crate::eval::MAX_STEPS)
            .types,
        Err(FeatureTypeIssue::WorkLimit)
    );
    integer(&mut r, "selected");
    let report = r.feature_type_report_with_budget(feature, 100);
    let expected = r.resolve_qualified("ScalarValues::Integer").unwrap();
    assert_eq!(report.types, Ok(vec![expected]));
    assert!(report.steps >= 100 && report.steps <= crate::eval::MAX_STEPS);
    assert_eq!(r.b.elements.len(), rows);
}

#[test]
fn parameter_membership_defaults_supply_direction_but_explicit_null_does_not() {
    for (kind, explicit_null, complete) in [
        ("FeatureMembership", false, false),
        ("ParameterMembership", false, true),
        ("ParameterMembership", true, false),
    ] {
        let mut r =
            fixture("function F {in selected:ScalarValues::Integer default=1;return result;}");
        let feature = r.resolve_qualified("F::selected").unwrap();
        let member = r.b.elements[feature.0].owning_relationship.unwrap();
        r.b.elements[member].ty = kind;
        r.b.elements[feature.0]
            .props
            .entries
            .make_mut()
            .retain(|(key, _)| key.name() != "direction");
        if explicit_null {
            r.b.set(feature.0, "direction", json!(null));
        }
        let typing = r
            .owned_relationships(feature)
            .into_iter()
            .find(|&e| r.element_type(e) == "FeatureTyping")
            .unwrap();
        r.b.set(typing.0, "isImplied", json!(true));
        r.ensure_implied();
        if complete {
            integer(&mut r, "F::selected");
        } else {
            let report = r.feature_type_report(feature);
            assert!(report.types.is_err(), "{kind}, {explicit_null}");
            assert_ne!(report.types, Err(FeatureTypeIssue::WorkLimit));
        }
    }
}

#[test]
fn behavior_predicate_and_step_parameters_use_the_same_valuation_typing_proof() {
    let mut r = fixture(
        "behavior B {in i:ScalarValues::Integer default=1;out o:ScalarValues::Integer default=1;inout io:ScalarValues::Integer default=1;} predicate P {in i:ScalarValues::Integer default=1;return r:ScalarValues::Boolean default=true;} step s {in i:ScalarValues::Integer default=1;} expr e {in i:ScalarValues::Integer default=1;return r:ScalarValues::Integer default=1;}",
    );
    r.ensure_implied();
    for name in ["B::i", "B::o", "B::io", "P::i", "s::i", "e::i", "e::r"] {
        integer(&mut r, name);
    }
    let result = r.resolve_qualified("P::r").unwrap();
    let boolean = r.resolve_qualified("ScalarValues::Boolean").unwrap();
    assert_eq!(r.feature_type_report(result).types, Ok(vec![boolean]));
}

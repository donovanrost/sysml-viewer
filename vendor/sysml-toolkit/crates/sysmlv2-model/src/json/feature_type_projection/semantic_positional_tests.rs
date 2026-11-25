use crate::{
    json::{ClosurePolicy, ElementRef, FeatureTypeIssue, ResolvedModel},
    model::{GraphFormat, Model},
    prepared::PreparedLibrary,
};
use serde_json::json;
use std::sync::{Arc, OnceLock};

fn raw_library() -> Model {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec-refs/SysML-v2-Release/sysml.library");
    let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
    assert_eq!(model.load_library_dir(&root).unwrap(), 94);
    model
}
fn library() -> Arc<PreparedLibrary> {
    static LIBRARY: OnceLock<Arc<PreparedLibrary>> = OnceLock::new();
    Arc::clone(LIBRARY.get_or_init(|| raw_library().prepare_library().unwrap()))
}
fn resolved(mut model: Model, source: &str) -> ResolvedModel {
    let added = model.add_source("positional-types.kerml", source);
    assert!(added.diagnostics.is_empty(), "{:?}", added.diagnostics);
    ResolvedModel::build(&model)
}
fn fixture(source: &str) -> ResolvedModel {
    let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
    library().install(&mut model).unwrap();
    resolved(model, source)
}
fn reference(r: &ResolvedModel) -> ElementRef {
    r.user_elements()
        .find(|&e| r.element_type(e) == "FeatureReferenceExpression")
        .unwrap()
}
fn generated(r: &mut ResolvedModel, source: ElementRef) -> ElementRef {
    r.ensure_implied();
    r.set_closure_policy(ClosurePolicy::Closure {
        include_implied: true,
    });
    ElementRef(
        r.b.semantic_ownership
            .as_ref()
            .unwrap()
            .result(source.0)
            .unwrap()
            .feature,
    )
}
fn exact(r: &mut ResolvedModel, result: ElementRef, name: &str) {
    let expected = r.resolve_qualified(name).unwrap();
    let rows = r.b.elements.len();
    let report = r.feature_type_report(result);
    assert_eq!(report.types, Ok(vec![expected]), "{name}: {report:?}");
    assert_eq!(
        r.property(result, "type").unwrap(),
        json!([{"@id":r.element_id(expected)}])
    );
    assert_eq!(r.b.elements.len(), rows);
}
const SOURCE: &str =
    "feature selectedMember:ScalarValues::Integer; feature queried=selectedMember;";

#[test]
fn generated_result_types_follow_the_shared_semantic_membership_certificate() {
    for (source, expected) in [
        (SOURCE, "ScalarValues::Integer"),
        (
            "feature selectedMember:ScalarValues::Boolean; feature queried=selectedMember;",
            "ScalarValues::Boolean",
        ),
        (
            "datatype Small specializes ScalarValues::Integer; feature selectedMember:Small; feature queried=selectedMember;",
            "Small",
        ),
        (
            "feature selectedMember:ScalarValues::Integer; feature narrowed subsets selectedMember; feature queried=narrowed;",
            "ScalarValues::Integer",
        ),
        (
            "class C {feature selectedMember:ScalarValues::Integer;} feature queried=C::selectedMember;",
            "ScalarValues::Integer",
        ),
        (
            "function F {in selectedMember:ScalarValues::Integer; return outputValue:ScalarValues::Integer;} feature queried=F::selectedMember;",
            "ScalarValues::Integer",
        ),
    ] {
        let mut r = fixture(source);
        let source = reference(&r);
        let result = generated(&mut r, source);
        assert!(!r.b.elem_scope.contains_key(&result.0));
        assert_eq!(r.expression_result_report(source).result, Ok(Some(result)));
        exact(&mut r, result, expected);
    }
}

#[test]
fn library_defaults_keep_actual_result_types_after_portion_dependencies() {
    let mut r = fixture(
        "function F {in x:ScalarValues::Integer default=1; return result:ScalarValues::Integer;} feature call=F();",
    );
    let call = r
        .user_elements()
        .find(|&e| r.element_type(e) == "InvocationExpression")
        .unwrap();
    r.ensure_implied();
    r.set_closure_policy(ClosurePolicy::Closure {
        include_implied: true,
    });
    let defaults = r
        .invocation_default_report(call)
        .selection
        .unwrap()
        .defaults;
    assert_eq!(defaults.len(), 10);
    let mut complete = 0;
    for default in defaults {
        let result = r
            .expression_result_report(default.value)
            .result
            .unwrap()
            .unwrap();
        let name = r.element_qualified_name(default.feature).unwrap();
        let expected = match name.as_str() {
            "F::x" => Some("ScalarValues::Integer"),
            "Performances::Performance::isDispatch"
            | "Occurrences::Occurrence::isRunToCompletion" => Some("ScalarValues::Boolean"),
            "Performances::Performance::dispatchScope"
            | "Performances::Performance::thisPerformance" => Some("Performances::Performance"),
            "Occurrences::Occurrence::portionOfLife"
            | "Occurrences::Occurrence::this"
            | "Occurrences::Occurrence::runToCompletionScope" => Some("Occurrences::Occurrence"),
            "Occurrences::Occurrence::localClock" => Some("Clocks::UniversalClockLife"),
            "Occurrences::Occurrence::incomingTransferSort" => {
                Some("Occurrences::IncomingTransferSort")
            }
            _ => panic!("Unexpected selected default {name}"),
        };
        if let Some(expected) = expected {
            exact(&mut r, result, expected);
            complete += 1;
        }
    }
    assert_eq!(complete, 10);
}

#[test]
fn result_type_queries_preserve_cumulative_budgets_and_retry_without_publication() {
    let mut r = fixture(SOURCE);
    let source = reference(&r);
    let result = generated(&mut r, source);
    let rows = r.b.elements.len();
    assert_eq!(
        r.feature_type_report_with_budget(result, crate::eval::MAX_STEPS)
            .types,
        Err(FeatureTypeIssue::WorkLimit)
    );
    exact(&mut r, result, "ScalarValues::Integer");
    let expected = r.resolve_qualified("ScalarValues::Integer").unwrap();
    let report = r.feature_type_report_with_budget(result, 100);
    assert_eq!(report.types, Ok(vec![expected]));
    assert!(report.steps > 100);
    assert_eq!(r.b.elements.len(), rows);
}

#[test]
fn result_type_identity_replays_from_raw_prepared_and_serialized_libraries() {
    let prepared = library();
    let decoded =
        Arc::new(PreparedLibrary::from_bytes(&prepared.to_bytes(147).unwrap(), 147).unwrap());
    let mut expected = None;
    for replay in [None, Some(prepared), Some(decoded)] {
        let model = if let Some(replay) = replay {
            let mut m = Model::with_graph_format(GraphFormat::CanonicalV3);
            replay.install(&mut m).unwrap();
            m
        } else {
            raw_library()
        };
        let mut r = resolved(model, SOURCE);
        let source = reference(&r);
        let result = generated(&mut r, source);
        exact(&mut r, result, "ScalarValues::Integer");
        let integer = r.resolve_qualified("ScalarValues::Integer").unwrap();
        let actual = (
            r.element_id(source),
            r.element_id(result),
            r.element_id(integer),
        );
        if let Some(expected) = expected {
            assert_eq!(actual, expected);
        } else {
            expected = Some(actual);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Damage {
    SourceOwner,
    ReferentOwner,
    Metadata,
    Direction,
    Composite,
    ResultOwner,
    ResultTarget,
}
const DAMAGE: [Damage; 7] = [
    Damage::SourceOwner,
    Damage::ReferentOwner,
    Damage::Metadata,
    Damage::Direction,
    Damage::Composite,
    Damage::ResultOwner,
    Damage::ResultTarget,
];
fn damage(r: &mut ResolvedModel, source: ElementRef, result: Option<ElementRef>, change: Damage) {
    let referent = r.resolve_qualified("selectedMember").unwrap();
    match change {
        Damage::SourceOwner => r.b.elements[source.0].owning_relationship = None,
        Damage::ReferentOwner => r.b.elements[referent.0].owning_relationship = None,
        Damage::Metadata => r.b.metadata_associations_incomplete = true,
        Damage::Direction => r.b.set(referent.0, "direction", json!("in")),
        Damage::Composite => r.b.set(referent.0, "isComposite", json!(true)),
        Damage::ResultOwner => r.b.elements[result.unwrap().0].owning_relationship = None,
        Damage::ResultTarget => {
            let result = result.unwrap();
            let redefinition = r
                .owned_relationships(result)
                .into_iter()
                .find(|&e| r.element_type(e) == "Redefinition")
                .unwrap();
            r.b.set(
                redefinition.0,
                "redefinedFeature",
                json!({"@id":"00000000-0000-0000-0000-000000000000"}),
            );
        }
    }
}
#[test]
fn malformed_sources_do_not_gain_complete_types_from_semantic_positions() {
    for change in DAMAGE.into_iter().take(5) {
        let mut r = fixture(SOURCE);
        let source = reference(&r);
        damage(&mut r, source, None, change);
        r.ensure_implied();
        let result =
            r.b.semantic_ownership
                .as_ref()
                .and_then(|v| v.result(source.0))
                .map(|v| ElementRef(v.feature));
        if let Some(result) = result {
            let report = r.feature_type_report(result);
            assert!(report.types.is_err(), "{change:?}");
            assert_ne!(report.types, Err(FeatureTypeIssue::WorkLimit), "{change:?}");
        } else {
            assert!(
                r.expression_result_report(source).result.is_err(),
                "{change:?}"
            );
        }
    }
}
#[test]
fn edited_sources_and_generated_rows_revoke_success() {
    for change in DAMAGE {
        let mut r = fixture(SOURCE);
        let source = reference(&r);
        let result = generated(&mut r, source);
        exact(&mut r, result, "ScalarValues::Integer");
        damage(&mut r, source, Some(result), change);
        // Arbitrary edits may exhaust rebuilding. They must never reuse success.
        assert!(r.feature_type_report(result).types.is_err(), "{change:?}");
    }
}

#[test]
fn unresolved_dependencies_are_distinct_from_closed_circular_typing_closures() {
    for source_text in [
        "feature selectedMember:Missing; feature queried=selectedMember;",
        "feature selectedMember:Missing subsets other; feature other subsets selectedMember; feature queried=selectedMember;",
    ] {
        let mut r = fixture(source_text);
        let source = reference(&r);
        r.ensure_implied();
        let result =
            r.b.semantic_ownership
                .as_ref()
                .and_then(|view| view.result(source.0))
                .map(|row| ElementRef(row.feature));
        if let Some(result) = result {
            let report = r.feature_type_report(result);
            assert!(report.types.is_err(), "{source_text}: {report:?}");
            assert_ne!(report.types, Err(FeatureTypeIssue::WorkLimit));
        } else {
            assert!(r.expression_result_report(source).result.is_err());
        }
    }
    // deriveFeatureType explicitly takes a closure that handles circular
    // relationships. An exact type does not certify whole-model validity.
    for (source_text, expected) in [
        (
            "feature selectedMember subsets other; feature other subsets selectedMember; feature queried=selectedMember;",
            "Base::Anything",
        ),
        (
            "feature selectedMember:ScalarValues::Integer subsets other; feature other subsets selectedMember; feature queried=selectedMember;",
            "ScalarValues::Integer",
        ),
    ] {
        let mut r = fixture(source_text);
        let source = reference(&r);
        let result = generated(&mut r, source);
        exact(&mut r, result, expected);
    }
}

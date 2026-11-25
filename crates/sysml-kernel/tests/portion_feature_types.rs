use sysmlv2_model::{
    json::{ClosurePolicy, FeatureTypeIssue, ResolvedModel},
    model::{GraphFormat, Model},
};
fn library(format: GraphFormat) -> Model {
    let mut model = Model::with_graph_format(format);
    let sources: Vec<serde_json::Value> =
        serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/library.json"))).unwrap();
    assert_eq!(sources.len(), 94);
    for source in sources {
        model.add_library_source(
            source["name"].as_str().unwrap(),
            source["text"].as_str().unwrap(),
        );
    }
    model
}
fn exact(graph: &mut ResolvedModel, name: &str, ty: &str) {
    let feature = graph.resolve_qualified(name).unwrap();
    let expected = graph.resolve_qualified(ty).unwrap();
    assert_eq!(
        graph.feature_type_report(feature).types,
        Ok(vec![expected]),
        "{name}"
    );
    assert_eq!(
        graph.property(feature, "type").unwrap(),
        serde_json::json!([{"@id":graph.element_id(expected)}]),
        "{name}"
    );
}
#[test]
fn portions_and_their_default_results_expose_exact_public_types() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let mut model = library(GraphFormat::CanonicalV3);
            let source = r#"
            class Owner {portion feature direct:Occurrences::Occurrence;}
            feature owner:Owner {portion feature nested:Occurrences::Occurrence;}
            feature inherited subsets owner {portion feature nested:Occurrences::Occurrence;}
            classifier Narrow specializes Owner;
            feature narrowed:Owner,Narrow {portion feature nested:Occurrences::Occurrence;}
            feature cyclic:Owner subsets peer {portion feature nested:Occurrences::Occurrence;}
            feature peer subsets cyclic;
            class ExternalTypingOwner {portion feature slot;}
            typing ExternalTypingOwner::slot typed by Occurrences::Occurrence;
            function F {in x:ScalarValues::Integer default=1;return result:ScalarValues::Integer;}
            feature call=F();
        "#;
            let parsed = model.add_source("portions.kerml", source);
            assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
            let mut graph = ResolvedModel::build(&model);
            let call = graph
                .user_elements()
                .find(|&e| graph.element_type(e) == "InvocationExpression")
                .unwrap();
            graph.implied_relationships(call);
            graph.set_closure_policy(ClosurePolicy::Closure {
                include_implied: true,
            });
            for name in [
                "Owner::direct",
                "owner::nested",
                "inherited::nested",
                "narrowed::nested",
                "cyclic::nested",
                "ExternalTypingOwner::slot",
                "Occurrences::Occurrence::portions",
                "Occurrences::Occurrence::self",
            ] {
                exact(&mut graph, name, "Occurrences::Occurrence");
            }
            let defaults = graph
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
            for (default, (name, ty)) in defaults.into_iter().zip(expected) {
                assert_eq!(
                    graph.element_qualified_name(default.feature).as_deref(),
                    Some(name)
                );
                let result = graph
                    .expression_result_report(default.value)
                    .result
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    graph.property(default.value, "result").unwrap(),
                    serde_json::json!({"@id":graph.element_id(result)})
                );
                let expected_type = graph.resolve_qualified(ty).unwrap();
                assert_eq!(
                    graph.feature_type_report(result).types,
                    Ok(vec![expected_type]),
                    "{name}"
                );
                assert_eq!(
                    graph.property(result, "type").unwrap(),
                    serde_json::json!([{"@id":graph.element_id(expected_type)}]),
                    "{name}"
                );
            }
            exact(
                &mut graph,
                "Occurrences::Occurrence::portionOfLife",
                "Occurrences::Life",
            );
            let mut model = library(GraphFormat::CanonicalV3);
            assert!(model
                .add_source(
                    "missing-owner.kerml",
                    "feature owner:Missing {portion feature nested:Occurrences::Occurrence;}"
                )
                .diagnostics
                .is_empty());
            let mut graph = ResolvedModel::build(&model);
            let f = graph.resolve_qualified("owner::nested").unwrap();
            graph.implied_relationships(f);
            let report = graph.feature_type_report(f);
            assert!(report.types.is_err());
            assert_ne!(report.types, Err(FeatureTypeIssue::WorkLimit));
        })
        .unwrap()
        .join()
        .unwrap();
}

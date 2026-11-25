use sysmlv2_model::{
    json::ResolvedModel,
    model::{GraphFormat, Model},
};

#[test]
fn effective_default_absence_and_owned_override_survive_full_library_construction() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
            let sources: Vec<serde_json::Value> =
                serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/library.json")))
                    .unwrap();
            assert_eq!(sources.len(), 94);
            for source in sources {
                model.add_library_source(
                    source["name"].as_str().unwrap(),
                    source["text"].as_str().unwrap(),
                );
            }
            assert!(model
                .add_source(
                    "defaults.kerml",
                    "class A {feature a default = 1;}
                    class B {feature b default = 1;}
                    class DistinctDefaults specializes A,B {
                        feature c redefines A::a,B::b;
                    }
                    class OwnedDefault specializes A,B {
                        feature c redefines A::a,B::b default = 3;
                    }
                    feature absent = new DistinctDefaults();
                    feature present = new OwnedDefault();",
                )
                .diagnostics
                .is_empty());
            let mut graph = ResolvedModel::build(&model);
            let calls: Vec<_> = graph
                .user_elements()
                .filter(|&e| graph.element_type(e) == "ConstructorExpression")
                .collect();
            assert_eq!(calls.len(), 2);
            graph.implied_relationships(calls[0]);
            let distinct = graph.resolve_qualified("DistinctDefaults").unwrap();
            let owned = graph.resolve_qualified("OwnedDefault").unwrap();
            let selected = graph.resolve_qualified("OwnedDefault::c").unwrap();
            // Both Classes also inherit these eight defaults from the pinned
            // Occurrences library. Absence for c must preserve all of them.
            let library_defaults: Vec<_> = [
                "portionOfLife",
                "this",
                "localClock",
                "isDispatch",
                "dispatchScope",
                "isRunToCompletion",
                "runToCompletionScope",
                "incomingTransferSort",
            ]
            .into_iter()
            .map(|name| {
                graph
                    .resolve_qualified(&format!("Occurrences::Occurrence::{name}"))
                    .unwrap()
            })
            .collect();
            for call in calls {
                let report = graph.constructor_result_report(call).result.unwrap();
                let defaults = report.defaults;
                let expected = if report.arguments.instantiated_type == distinct {
                    // Equal literals still belong to distinct FeatureValues:
                    // c has no effective default under KerML 8.4.4.11.
                    library_defaults.clone()
                } else {
                    assert_eq!(report.arguments.instantiated_type, owned);
                    std::iter::once(selected)
                        .chain(library_defaults.iter().copied())
                        .collect()
                };
                assert_eq!(
                    defaults.iter().map(|d| d.feature).collect::<Vec<_>>(),
                    expected
                );
                assert_eq!(
                    defaults
                        .iter()
                        .map(|d| d.feature_with_value)
                        .collect::<Vec<_>>(),
                    expected
                );
                // Immutable owned_members is source navigation. Generated
                // bindings are reached through the semantic relationship API.
                let members: Vec<_> = graph
                    .implied_relationships(report.arguments.result)
                    .into_iter()
                    .filter_map(|relationship| graph.membership_member(relationship))
                    .collect();
                let bindings: Vec<_> = members
                    .into_iter()
                    .filter(|&e| graph.element_type(e) == "BindingConnector")
                    .collect();
                assert_eq!(bindings.len(), defaults.len());
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

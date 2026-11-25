use sysmlv2_model::{
    json::{ClosurePolicy, ResolvedModel},
    model::{GraphFormat, Model},
};

#[test]
fn calls_and_triggers_publish_default_bindings_with_exact_public_endpoints() {
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
            assert!(model.add_source("calls.kerml", "function F {in a default=1;return r;} feature missing=F();feature supplied=F(2);").diagnostics.is_empty());
            assert!(model.add_source("triggers.sysml", "attribute d:ISQ::DurationValue;attribute t:Time::TimeInstantValue;attribute c:ScalarValues::Boolean;action delay accept after d;action instant accept at t;action change accept when c;").diagnostics.is_empty());
            let mut graph = ResolvedModel::build(&model);
            let calls: Vec<_> = graph.user_elements().filter(|&e| matches!(graph.element_type(e), "InvocationExpression" | "TriggerInvocationExpression")).collect();
            assert_eq!(calls.len(), 5);
            graph.implied_relationships(calls[0]);
            graph.set_closure_policy(ClosurePolicy::Closure {include_implied: true});
            let mut checked_default_results = 0;
            for call in calls {
                let selection = graph.invocation_default_report(call).selection.unwrap();
                let callee = graph.element_qualified_name(selection.arguments.callee).unwrap();
                let expected = match callee.as_str() {
                    "F" if selection.arguments.bindings.is_empty() => 10,
                    "F" => 9,
                    "Triggers::TriggerWhen" => 10,
                    "Triggers::TriggerAt" | "Triggers::TriggerAfter" => 11,
                    other => panic!("unexpected callee {other}"),
                };
                assert_eq!(selection.defaults.len(), expected);
                for default in &selection.defaults {
                    let result = graph.expression_result_report(default.value).result
                        .unwrap_or_else(|issue| panic!("{callee}: default {:?}: {issue:?}", graph.element_qualified_name(default.feature)))
                        .expect("Every selected default in this fixture has a result");
                    assert_eq!(graph.property(default.value, "result").unwrap(), serde_json::json!({"@id":graph.element_id(result)}));
                    if graph.element_type(default.value) == "FeatureReferenceExpression" {
                        assert_eq!(graph.owner(result), Some(default.value));
                    }
                    checked_default_results += 1;
                }
                let relationships: Vec<_> = graph.implied_relationships(call).into_iter()
                    .filter(|&rel| graph.element_type(rel) == "OwningMembership").collect();
                let members: Vec<_> = relationships.into_iter()
                    .filter_map(|rel| graph.membership_member(rel)).collect();
                let bindings: Vec<_> = members.into_iter()
                    .filter(|&member| graph.element_type(member) == "BindingConnector").collect();
                assert_eq!(bindings.len(), expected);
                let mut actual = Vec::new();
                for binding in bindings {
                    assert_eq!(graph.owner(binding), Some(call));
                    let ends = graph.owned_features(binding);
                    assert_eq!(ends.len(), 2);
                    let targets: Vec<_> = ends.into_iter().map(|end| {
                        let references: Vec<_> = graph.owned_relationships(end).into_iter()
                            .filter(|&rel| graph.element_type(rel) == "ReferenceSubsetting").collect();
                        assert_eq!(references.len(), 1);
                        assert_eq!(graph.property(references[0], "subsettingFeature").unwrap(), serde_json::json!({"@id":graph.element_id(end)}));
                        graph.property(references[0], "referencedFeature").unwrap()
                    }).collect();
                    actual.push(targets);
                }
                let expected: Vec<_> = selection.defaults.iter().map(|default| vec![
                    serde_json::json!({"@id":graph.element_id(default.feature_with_value)}),
                    serde_json::json!({"@id":graph.element_id(default.value)}),
                ]).collect();
                assert_eq!(actual, expected, "{callee}");
            }
            assert_eq!(checked_default_results, 51);
        })
        .unwrap()
        .join()
        .unwrap();
}

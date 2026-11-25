use sysmlv2_model::{
    json::{ClosurePolicy, Derived, ResolvedModel},
    model::{GraphFormat, Model},
};

#[test]
fn invocations_own_distinct_results_redefining_the_actual_library_callee() {
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
            assert!(model.add_source("functions.kerml", "function DelayFn {return result:ISQ::DurationValue;} function InstantFn {return result:Time::TimeInstantValue;} function ReadyFn {return result:ScalarValues::Boolean;}").diagnostics.is_empty());
            assert!(model.add_source("calls.sysml", "action d accept after DelayFn(); action t accept at InstantFn(); action b accept when ReadyFn(); action c accept when 1 < 2; action n accept after 1 + 2;").diagnostics.is_empty());
            let mut graph = ResolvedModel::build(&model);
            let calls: Vec<_> = graph.user_elements().filter(|&e| matches!(graph.element_type(e), "InvocationExpression" | "OperatorExpression" | "TriggerInvocationExpression")).collect();
            assert_eq!(calls.len(), 10);
            graph.implied_relationships(calls[0]);
            graph.set_closure_policy(ClosurePolicy::Closure {include_implied: true});
            for call in calls {
                let results: Vec<_> = graph.owned_features(call).into_iter().filter(|&f| graph.owning_membership_type(f) == Some("ReturnParameterMembership")).collect();
                assert_eq!(results.len(), 1, "{}", graph.element_type(call));
                let result = results[0];
                assert_eq!(graph.owner(result), Some(call));
                let args = graph.invocation_binding_report(call).arguments.unwrap();
                assert!(args.bindings.iter().all(|b| b.parameter != result));
                let callee_result = graph.function_result_report(args.callee).result.unwrap().unwrap();
                assert_ne!(result, callee_result);
                // Redefinition endpoints are stored metamodel properties, not
                // derived ones. Read and validate both through the public API.
                let source = serde_json::json!({"@id":graph.element_id(result)});
                let target = serde_json::json!({"@id":graph.element_id(callee_result)});
                assert!(graph.implied_relationships(result).into_iter().any(|relationship| {
                    graph.element_type(relationship) == "Redefinition"
                        && graph.property(relationship, "redefiningFeature") == Ok(source.clone())
                        && graph.property(relationship, "redefinedFeature") == Ok(target.clone())
                }), "{} {}: missing owned result redefinition to {}", graph.element_type(call), graph.element_id(call), graph.element_id(callee_result));
                assert!(matches!(graph.derived(call, "result"), Derived::Value(value) if value.element() == Some(result)));
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

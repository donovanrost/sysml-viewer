use sysmlv2_model::{
    json::{ClosurePolicy, ResolvedModel},
    model::{GraphFormat, Model},
};

#[test]
fn ordinary_features_and_library_defaults_expose_declared_types() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
            let sources: Vec<serde_json::Value> = serde_json::from_str(include_str!(concat!(
                env!("OUT_DIR"),
                "/library.json"
            )))
            .unwrap();
            assert_eq!(sources.len(), 94);
            for source in sources {
                model.add_library_source(
                    source["name"].as_str().unwrap(),
                    source["text"].as_str().unwrap(),
                );
            }
            let parsed = model.add_source(
                "ordinary-features.kerml",
                r#"
                feature namespaceControl:ScalarValues::Integer;
                feature namespaceValue:ScalarValues::Integer default=1;
                classifier C {feature control:ScalarValues::Integer;feature valued:ScalarValues::Integer default=1;}
                behavior B {feature control:ScalarValues::Integer;feature valued:ScalarValues::Integer default=1;}
                function F {feature control:ScalarValues::Integer;feature valued:ScalarValues::Integer default=1;return result;}
                predicate P {feature control:ScalarValues::Integer;feature valued:ScalarValues::Integer default=1;return result:ScalarValues::Boolean;}
                step s {feature control:ScalarValues::Integer;feature valued:ScalarValues::Integer default=1;}
                expr e {feature control:ScalarValues::Integer;feature valued:ScalarValues::Integer default=1;return result;}
                feature outer:Base::Anything {feature control:ScalarValues::Integer;feature valued:ScalarValues::Integer default=1;}
                classifier A {feature forward:A;feature backward:A inverse of forward;}
                function WithDefault {in x:ScalarValues::Integer default=1;return result:ScalarValues::Integer;}
                feature call=WithDefault();
            "#,
            );
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
                "namespaceControl", "namespaceValue", "C::control", "C::valued",
                "B::control", "B::valued", "F::control", "F::valued", "P::control",
                "P::valued", "s::control", "s::valued", "e::control", "e::valued",
                "outer::control", "outer::valued", "A::forward", "A::backward",
            ] {
                let feature = graph.resolve_qualified(name).unwrap();
                let expected = if name.starts_with("A::") { "A" } else { "ScalarValues::Integer" };
                let expected = graph.resolve_qualified(expected).unwrap();
                assert_eq!(graph.feature_type_report(feature).types, Ok(vec![expected]), "{name}");
                assert_eq!(graph.property(feature, "type").unwrap(), serde_json::json!([{"@id":graph.element_id(expected)}]), "{name}");
            }
            for name in ["B", "F", "P", "s", "e"] {
                let feature = graph.resolve_qualified(&format!("{name}::valued")).unwrap();
                let owner = graph.resolve_qualified(name).unwrap();
                let roles = graph.type_feature_report(owner).projections.unwrap();
                assert!(roles.features.contains(&feature));
                assert!(!roles.inputs.contains(&feature));
                assert!(!roles.outputs.contains(&feature));
            }
            let defaults = graph.invocation_default_report(call).selection.unwrap().defaults;
            assert_eq!(defaults.len(), 10);
            for default in defaults {
                let name = graph.element_qualified_name(default.feature).unwrap();
                let expected = match name.as_str() {
                    "WithDefault::x" => "ScalarValues::Integer",
                    "Performances::Performance::isDispatch" | "Occurrences::Occurrence::isRunToCompletion" => "ScalarValues::Boolean",
                    "Performances::Performance::dispatchScope" | "Occurrences::Occurrence::this" | "Occurrences::Occurrence::runToCompletionScope" => "Occurrences::Occurrence",
                    "Performances::Performance::thisPerformance" => "Performances::Performance",
                    "Occurrences::Occurrence::portionOfLife" => "Occurrences::Life",
                    "Occurrences::Occurrence::localClock" => "Clocks::Clock",
                    "Occurrences::Occurrence::incomingTransferSort" => "Occurrences::IncomingTransferSort",
                    _ => panic!("Unexpected default: {name}"),
                };
                let expected = graph.resolve_qualified(expected).unwrap();
                assert_eq!(graph.feature_type_report(default.feature).types, Ok(vec![expected]), "{name}");
                assert_eq!(graph.property(default.feature, "type").unwrap(), serde_json::json!([{"@id":graph.element_id(expected)}]), "{name}");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

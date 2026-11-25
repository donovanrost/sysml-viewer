use sysmlv2_model::{
    json::{ClosurePolicy, ResolvedModel},
    model::{GraphFormat, Model},
};

#[test]
fn values_preserve_declared_types_through_the_checked_report_and_property() {
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
            let source = r#"
                function F {
                    in a:ScalarValues::Integer default=1;
                    out b:ScalarValues::Integer default=1;
                    inout c:ScalarValues::Integer=1;
                    return r:ScalarValues::Integer default=1;
                }
                feature typedFeature:ScalarValues::Integer default=1;
                feature subsetFeature subsets typedFeature default=1;
                class A {feature a:ScalarValues::Integer;}
                class B specializes A {feature b redefines A::a default=1;}
                function Child specializes F {in a redefines F::a default=1;}
                feature mismatch:ScalarValues::Integer="text";
                feature unresolved:ScalarValues::Integer default=missing;
                feature inferred default=1;
            "#;
            let parsed = model.add_source("valued-types.kerml", source);
            assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
            let mut graph = ResolvedModel::build(&model);
            let integer = graph.resolve_qualified("ScalarValues::Integer").unwrap();
            graph.implied_relationships(integer);
            graph.set_closure_policy(ClosurePolicy::Closure {
                include_implied: true,
            });
            for name in [
                "F::a",
                "F::b",
                "F::c",
                "F::r",
                "typedFeature",
                "subsetFeature",
                "B::b",
                "Child::a",
                "mismatch",
                "unresolved",
            ] {
                let feature = graph.resolve_qualified(name).unwrap();
                assert_eq!(
                    graph.feature_type_report(feature).types,
                    Ok(vec![integer]),
                    "{name}"
                );
                assert_eq!(
                    graph.property(feature, "type").unwrap(),
                    serde_json::json!([{"@id":graph.element_id(integer)}]),
                    "{name}"
                );
            }
            let inferred = graph.resolve_qualified("inferred").unwrap();
            assert!(graph.feature_type_report(inferred).types.is_err());
            assert!(graph.property(inferred, "type").is_err());
        })
        .unwrap()
        .join()
        .unwrap();
}

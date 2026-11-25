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

#[test]
fn chained_features_expose_exact_static_types_through_public_queries() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            for format in [GraphFormat::LegacyV2, GraphFormat::CanonicalV3] {
                let mut model = library(format);
                let source = r#"
                    datatype Small specializes ScalarValues::Integer;
                    classifier Nested {feature leaf:Small;}
                    classifier Holder {feature next:ScalarValues::Integer;feature nested:Nested;feature compound chains nested.leaf;}
                    feature root:Holder;
                    feature simple chains root.next;
                    feature deep chains root.nested.leaf;
                    feature narrow:Small chains root.next;
                    feature inherited subsets simple;
                    feature nestedChain chains root.compound;
                    behavior BehaviorOwner {feature scoped chains root.next;}
                    class Parent {feature original chains root.next;}
                    class Child specializes Parent {feature replaced redefines Parent::original;}
                    classifier Node {feature link:Node;feature leaf:Small;}
                    feature node:Node;
                    feature repeated chains node.link.link.leaf;
                "#;
                if format == GraphFormat::LegacyV2 {
                    // Legacy construction omits unconditional owned-typing bases.
                    // A chain cannot certify a type when its final Feature lacks
                    // the required Base::dataValues path. Keep that refusal visible.
                    let parsed = model.add_source("chained-types.kerml", source);
                    assert!(parsed.diagnostics.is_empty());
                    let mut graph = ResolvedModel::build(&model);
                    let chain = graph.resolve_qualified("simple").unwrap();
                    graph.implied_relationships(chain);
                    for name in ["Holder::next", "simple"] {
                        let feature = graph.resolve_qualified(name).unwrap();
                        let report = graph.feature_type_report(feature);
                        assert_eq!(report.types, Err(FeatureTypeIssue::MissingRequiredFamilies));
                    }
                    let feature = graph.resolve_qualified("Base::Anything::self").unwrap();
                    let expected = graph.resolve_qualified("Base::Anything").unwrap();
                    assert_eq!(graph.feature_type_report(feature).types, Ok(vec![expected]));
                    model = library(format);
                }
                // Canonical input relies on implicit construction. The legacy
                // fixture supplies its missing normative paths explicitly, so
                // both representations exercise the same complete chain reader.
                let source = if format == GraphFormat::LegacyV2 {
                    source.replace(":ScalarValues::Integer", ":ScalarValues::Integer subsets Base::dataValues")
                        .replace(":Small", ":Small subsets Base::dataValues")
                } else { source.to_owned() };
                let parsed = model.add_source("chained-types.kerml", &source);
                assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
                let mut graph = ResolvedModel::build(&model);
                let root = graph.resolve_qualified("root").unwrap();
                graph.implied_relationships(root);
                graph.set_closure_policy(ClosurePolicy::Closure { include_implied: true });
                for (name, type_name) in [
                    ("simple", "ScalarValues::Integer"),
                    ("deep", "Small"),
                    ("narrow", "Small"),
                    ("inherited", "ScalarValues::Integer"),
                    ("nestedChain", "Small"),
                    ("BehaviorOwner::scoped", "ScalarValues::Integer"),
                    ("Parent::original", "ScalarValues::Integer"),
                    ("Child::replaced", "ScalarValues::Integer"),
                    ("repeated", "Small"),
                    ("Base::Anything::self", "Base::Anything"),
                ] {
                    let feature = graph.resolve_qualified(name).unwrap();
                    let expected = graph.resolve_qualified(type_name).unwrap();
                    let report = graph.feature_type_report(feature);
                    assert_eq!(report.types, Ok(vec![expected]), "{format:?}: {name}");
                    assert_eq!(graph.property(feature, "type").unwrap(), serde_json::json!([{"@id":graph.element_id(expected)}]), "{format:?}: {name}");
                }
            }
            let mut model = library(GraphFormat::CanonicalV3);
            let parsed = model.add_source(
                "unresolved-chain.kerml",
                "classifier Holder {feature next:Missing;} feature root:Holder;feature chain chains root.next;",
            );
            assert!(parsed.diagnostics.is_empty());
            let mut graph = ResolvedModel::build(&model);
            let feature = graph.resolve_qualified("chain").unwrap();
            graph.implied_relationships(feature);
            graph.set_closure_policy(ClosurePolicy::Closure { include_implied: true });
            let report = graph.feature_type_report(feature);
            assert!(report.types.is_err());
            assert_ne!(report.types, Err(FeatureTypeIssue::WorkLimit));
        })
        .unwrap()
        .join()
        .unwrap();
}

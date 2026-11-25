use sysmlv2_model::{
    json::{ClosurePolicy, ElementRef, FeatureTypeIssue, ResolvedModel},
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

fn exact(r: &mut ResolvedModel, name: &str, expected: &str) -> ElementRef {
    let feature = r.resolve_qualified(name).unwrap();
    let ty = r.resolve_qualified(expected).unwrap();
    assert_eq!(r.feature_type_report(feature).types, Ok(vec![ty]), "{name}");
    assert_eq!(
        r.property(feature, "type").unwrap(),
        serde_json::json!([{"@id":r.element_id(ty)}])
    );
    feature
}
#[test]
fn authored_namespace_bindings_preserve_public_parameter_queries() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
                let mut model = library(format);
                let source = r#"
                    function F {
                        member expr e : Performances::LiteralIntegerEvaluation;
                        in x : ScalarValues::Integer;
                        return result : ScalarValues::Integer;
                    }
                    expr actual : F {
                        in local redefines F::x;
                        member feature right chains F::e.result;
                        member binding factored of local = right;
                        member binding inlineChain of local = F::e.result;
                    }
                "#;
                if format == GraphFormat::LegacyV2 {
                    // Legacy construction does not supply unconditional owned
                    // data-value typing bases. Preserve that independent limit.
                    assert!(model.add_source("binding-members.kerml", source).diagnostics.is_empty());
                    let mut graph = ResolvedModel::build(&model);
                    let owner = graph.resolve_qualified("actual").unwrap();
                    graph.implied_relationships(owner);
                    let local = graph.resolve_qualified("actual::local").unwrap();
                    assert_eq!(graph.type_input_report(owner).inputs, Ok(vec![local]));
                    for name in ["F::x", "actual::local", "actual::right"] {
                        let member = graph.resolve_qualified(name).unwrap();
                        assert_eq!(graph.feature_type_report(member).types,
                            Err(FeatureTypeIssue::MissingRequiredFamilies));
                    }
                    model = library(format);
                }
                // Canonical uses ordinary source and implicit construction.
                // Legacy supplies the missing type paths explicitly, including
                // the selected expression result, to test the same exact query.
                let source = if format == GraphFormat::LegacyV2 {
                    // The literal library's inherited result also lacks its
                    // unconditional legacy bases. Use an authored complete
                    // signature; keep the original literal-result refusal above.
                    format!("function ExplicitValue {{return result : ScalarValues::Integer;}}\n{source}")
                        .replace("Performances::LiteralIntegerEvaluation", "ExplicitValue")
                        .replace(": ScalarValues::Integer", ": ScalarValues::Integer subsets Base::dataValues")
                } else { source.to_owned() };
                let parsed = model.add_source("binding-members.kerml", &source);
                assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
                let mut r = ResolvedModel::build(&model);
                let owner = r.resolve_qualified("actual").unwrap();
                r.implied_relationships(owner);
                r.set_closure_policy(ClosurePolicy::Closure {
                    include_implied: true,
                });
                let local = exact(&mut r, "actual::local", "ScalarValues::Integer");
                exact(&mut r, "actual::right", "ScalarValues::Integer");
                assert_eq!(r.type_input_report(owner).inputs, Ok(vec![local]));
                let projection = r.type_feature_report(owner).projections.unwrap();
                for name in ["actual::factored", "actual::inlineChain"] {
                    let binding = r.resolve_qualified(name).unwrap();
                    assert!(projection.members.as_ref().unwrap().contains(&binding));
                    assert!(!projection.features.contains(&binding));
                    assert!(!projection.inputs.contains(&binding));
                    assert!(!projection.outputs.contains(&binding));
                    assert_eq!(
                        r.property(binding, "owningType"),
                        Ok(serde_json::json!(null))
                    );
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

use sysmlv2_model::{
    json::{ClosurePolicy, ElementRef, ResolvedModel},
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
fn type_namespace_members_preserve_public_types_and_membership_roles() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let mut model = library(GraphFormat::CanonicalV3);
            let parsed = model.add_source(
                "namespace-members.kerml",
                r#"
            function F {
                member expr e:Performances::LiteralIntegerEvaluation;
                member feature scalar:ScalarValues::Integer default=2;
                member feature path chains e.result;
                in x:ScalarValues::Integer default=1;
                return result:ScalarValues::Integer;
            }
            feature call=F();
        "#,
            );
            assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
            let mut r = ResolvedModel::build(&model);
            let f = r.resolve_qualified("F").unwrap();
            r.implied_relationships(f);
            r.set_closure_policy(ClosurePolicy::Closure {
                include_implied: true,
            });
            let e = exact(&mut r, "F::e", "Performances::LiteralIntegerEvaluation");
            let scalar = exact(&mut r, "F::scalar", "ScalarValues::Integer");
            let path = exact(&mut r, "F::path", "ScalarValues::Integer");
            let result = r.expression_result_report(e).result.unwrap().unwrap();
            let integer = r.resolve_qualified("ScalarValues::Integer").unwrap();
            assert_eq!(r.feature_type_report(result).types, Ok(vec![integer]));
            let members = r.type_feature_report(f).projections.unwrap();
            for member in [e, scalar, path] {
                assert_eq!(
                    r.property(member, "owningType"),
                    Ok(serde_json::json!(null))
                );
                assert!(members.members.as_ref().unwrap().contains(&member));
                assert!(!members.features.contains(&member));
                assert!(!members.inputs.contains(&member));
                assert!(!members.outputs.contains(&member));
            }
            let x = r.resolve_qualified("F::x").unwrap();
            assert_eq!(members.inputs, [x]);
            let call = r
                .user_elements()
                .find(|&e| r.element_type(e) == "InvocationExpression")
                .unwrap();
            let defaults = r
                .invocation_default_report(call)
                .selection
                .unwrap()
                .defaults;
            let names: Vec<_> = defaults
                .iter()
                .map(|d| r.element_qualified_name(d.feature).unwrap())
                .collect();
            assert_eq!(
                names,
                [
                    "F::x",
                    "Performances::Performance::isDispatch",
                    "Performances::Performance::dispatchScope",
                    "Performances::Performance::thisPerformance",
                    "Occurrences::Occurrence::portionOfLife",
                    "Occurrences::Occurrence::this",
                    "Occurrences::Occurrence::localClock",
                    "Occurrences::Occurrence::isRunToCompletion",
                    "Occurrences::Occurrence::runToCompletionScope",
                    "Occurrences::Occurrence::incomingTransferSort"
                ]
            );
            let mut model = library(GraphFormat::CanonicalV3);
            assert!(model
                .add_source(
                    "unresolved-member.kerml",
                    "function F {member feature broken:Missing;}"
                )
                .diagnostics
                .is_empty());
            let mut r = ResolvedModel::build(&model);
            let f = r.resolve_qualified("F::broken").unwrap();
            r.implied_relationships(f);
            assert!(r.feature_type_report(f).types.is_err());
        })
        .unwrap()
        .join()
        .unwrap();
}

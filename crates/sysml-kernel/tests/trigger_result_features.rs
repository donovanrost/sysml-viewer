use std::collections::HashSet;
use sysmlv2_model::{
    json::{ClosurePolicy, Derived, ElementRef, ResolvedModel, TypeInputIssue},
    model::{GraphFormat, Model},
};

fn one(graph: &mut ResolvedModel, owner: ElementRef, property: &str) -> ElementRef {
    let Derived::Value(value) = graph.derived(owner, property) else {
        panic!("Missing {property}")
    };
    value.element().unwrap()
}

#[test]
fn trigger_validators_use_quantity_redefinitions_and_canonical_diagnostic_ids() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
            let sources: Vec<serde_json::Value> =
                serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/library.json")))
                    .unwrap();
            for source in sources {
                model.add_library_source(
                    source["name"].as_str().unwrap(),
                    source["text"].as_str().unwrap(),
                );
            }
            model.add_source(
                "positive.sysml",
                include_str!(
                    "../../../tests/conformance/fixtures/trigger-arguments-positive.sysml"
                ),
            );
            let negatives = include_str!(
                "../../../tests/conformance/fixtures/trigger-arguments-negative.sysml"
            );
            assert!(model
                .add_source("negative.sysml", negatives)
                .diagnostics
                .is_empty());
            let mut graph = ResolvedModel::build(&model);
            // The validator entry point must prepare the same construction itself.
            let diagnostics = sysmlv2_model::check::validate_semantics_with(&mut graph, &model);
            let trigger_diagnostics: Vec<_> = diagnostics
                .iter()
                .filter(|(_, d)| {
                    d.code
                        .is_some_and(|c| c.starts_with("validateTriggerInvocation"))
                })
                .collect();
            for (name, kind) in [
                ("wrongUnit", "After"),
                ("unrelatedUnit", "After"),
                ("lengthValue", "After"),
                ("scalarValue", "After"),
                ("booleanAfter", "After"),
                ("booleanAt", "At"),
                ("integerAt", "At"),
                ("integerWhen", "When"),
            ] {
                let accept = graph
                    .resolve_qualified(&format!("TriggerWrong::Reception::{name}"))
                    .unwrap();
                let trigger = one(&mut graph, accept, "payloadArgument");
                let report = graph.trigger_argument_report(trigger);
                assert_eq!(report.valid, Ok(false), "{name}: {} steps", report.steps);
                let rule = format!("validateTriggerInvocationExpression{kind}Argument");
                assert_eq!(report.rule, Some(rule.as_str()));
                let start = negatives.find(&format!("action {name} ")).unwrap();
                let end = start + negatives[start..].find(';').unwrap();
                assert!(
                    trigger_diagnostics
                        .iter()
                        .any(|(_, d)| d.code == Some(rule.as_str())
                            && d.span.start as usize >= start
                            && d.span.end as usize <= end),
                    "missing {name}: {trigger_diagnostics:?}"
                );
            }
            assert_eq!(trigger_diagnostics.len(), 8, "{trigger_diagnostics:?}");
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn generated_trigger_argument_results_inherit_quantity_reference_features() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
                let mut model = Model::with_graph_format(format);
                let sources: Vec<serde_json::Value> =
                    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/library.json")))
                        .unwrap();
                for source in sources {
                    model.add_library_source(
                        source["name"].as_str().unwrap(),
                        source["text"].as_str().unwrap(),
                    );
                }
                assert!(model
                    .add_source(
                        "triggers.sysml",
                        include_str!(
                            "../../../tests/conformance/fixtures/trigger-arguments-positive.sysml"
                        )
                    )
                    .diagnostics
                    .is_empty());
                let mut graph = ResolvedModel::build(&model);
                graph.set_closure_policy(ClosurePolicy::Closure {
                    include_implied: true,
                });
                let package = graph.resolve_qualified("TriggerArguments").unwrap();
                graph.implied_relationships(package);
                if format == GraphFormat::CanonicalV3 {
                    for (name, function, input, count) in [
                        ("independent", "TriggerAfter", "delay", 4),
                        ("inherited", "TriggerAfter", "delay", 4),
                        ("ordinary", "TriggerAfter", "delay", 4),
                        ("absolute", "TriggerAt", "timeInstant", 4),
                        ("conditional", "TriggerWhen", "condition", 3),
                        ("literal", "TriggerWhen", "condition", 3),
                    ] {
                        let accept = graph.resolve_qualified(&format!("TriggerArguments::Reception::{name}")).unwrap();
                        let trigger = one(&mut graph, accept, "payloadArgument");
                        let callee = graph.resolve_qualified(&format!("Triggers::{function}")).unwrap();
                        let input = graph.resolve_qualified(&format!("Triggers::{function}::{input}")).unwrap();
                        let validation = graph.trigger_argument_report(trigger);
                        assert_eq!(validation.valid, Ok(true), "{name}: {} steps", validation.steps);
                        let report = graph.invocation_binding_report(trigger);
                        let args = report.arguments.unwrap_or_else(|e| panic!("{name}: {e:?} after {} steps",report.steps));
                        assert_eq!(args.callee, callee);
                        assert_eq!(args.inputs.len(), count);
                        assert_eq!(args.bindings.len(), 1);
                        assert_eq!(args.bindings[0].input, input);
                        assert_eq!(graph.property(trigger, "argument"), Ok(serde_json::json!([{"@id":graph.element_id(args.bindings[0].value)}])));
                        assert!(graph.explicit_specializations(args.bindings[0].parameter).is_empty());
                        let source = if matches!(name, "conditional" | "literal") {
                            one(&mut graph, args.bindings[0].value, "referent")
                        } else {
                            args.bindings[0].value
                        };
                        let selected = graph.expression_result_report(source);
                        let result = selected.result.unwrap_or_else(|issue| panic!("{name}: result selection: {issue:?} after {} steps", selected.steps)).unwrap();
                        assert_eq!(graph.property(source, "result"), Ok(serde_json::json!({"@id":graph.element_id(result)})));
                        if name != "literal" {
                            assert_eq!(graph.owner(result), Some(source));
                            assert!(graph.explicit_specializations(result).is_empty());
                        }
                    }
                }
                let conditional_accept = graph.resolve_qualified("TriggerArguments::Reception::conditional").unwrap();
                let conditional_trigger = one(&mut graph, conditional_accept, "payloadArgument");
                let Derived::Value(arguments) = graph.derived(conditional_trigger, "argument") else {panic!("Missing conditional argument")};
                let reference = one(&mut graph, arguments.elements()[0], "referent");
                assert_eq!(graph.element_type(reference), "FeatureReferenceExpression");
                let enclosed = graph.resolve_qualified("Performances::Performance::enclosedPerformances").unwrap();
                assert!(graph.conforms_with_implied(reference, enclosed));
                let result = one(&mut graph, reference, "result");
                let report = graph.type_feature_report(result);
                if format == GraphFormat::CanonicalV3 {
                    let checked = report.projections.unwrap_or_else(|issue| panic!("nested when reference: {issue:?} after {} steps", report.steps));
                    let data_self = graph.resolve_qualified("Base::DataValue::self").unwrap();
                    let things_that = graph.resolve_qualified("Base::things::that").unwrap();
                    assert_eq!(checked.features.len(), 2);
                    assert!(checked.features.contains(&data_self));
                    assert!(checked.features.contains(&things_that));
                } else {
                    assert_eq!(report.projections, Err(TypeInputIssue::IncompleteProvider));
                }
                let literal_accept = graph.resolve_qualified("TriggerArguments::Reception::literal").unwrap();
                let literal_trigger = one(&mut graph, literal_accept, "payloadArgument");
                let Derived::Value(arguments) = graph.derived(literal_trigger, "argument") else {panic!("Missing literal argument")};
                let literal = one(&mut graph, arguments.elements()[0], "referent");
                let selected = graph.expression_result_report(literal);
                let result = selected.result.unwrap_or_else(|issue| panic!("{format:?}: checked literal result: {issue:?} after {} steps", selected.steps)).unwrap();
                assert_eq!(graph.property(literal, "result"), Ok(serde_json::json!({"@id":graph.element_id(result)})));
                let predicate = graph.owner(result).unwrap();
                assert_eq!(graph.element_type(predicate), "Predicate");
                assert_eq!(graph.function_result_report(predicate).result, Ok(Some(result)));
                assert_eq!(graph.property(predicate, "result"), Ok(serde_json::json!({"@id":graph.element_id(result)})));
                let report = graph.type_feature_report(result);
                if format == GraphFormat::LegacyV2 {
                    let data_values = graph.resolve_qualified("Base::dataValues").unwrap();
                    assert!(!graph.conforms_with_implied(result, data_values));
                    assert_eq!(report.projections, Err(TypeInputIssue::IncompleteProvider));
                } else {
                    let features = report.projections.unwrap_or_else(|issue| panic!("{format:?}: Boolean literal result: {issue:?} after {} steps",report.steps));
                    // DataValue explicitly redefines Anything::self. The Boolean
                    // return retains that specialization through ScalarValue.
                    let data_self = graph.resolve_qualified("Base::DataValue::self").unwrap();
                    let things_that = graph.resolve_qualified("Base::things::that").unwrap();
                    assert_eq!(features.features.len(), 2);
                    assert!(features.features.contains(&data_self));
                    assert!(features.features.contains(&things_that));
                }
                for (name, reference) in [
                    ("independent", "TriggerArguments::CustomDuration::mRef"),
                    ("inherited", "TriggerArguments::CustomDuration::mRef"),
                    ("ordinary", "ISQ::DurationValue::mRef"),
                ] {
                    let accept = graph
                        .resolve_qualified(&format!("TriggerArguments::Reception::{name}"))
                        .unwrap();
                    let trigger = one(&mut graph, accept, "payloadArgument");
                    let Derived::Value(arguments) = graph.derived(trigger, "argument") else {
                        panic!("Missing argument")
                    };
                    let result = one(&mut graph, arguments.elements()[0], "result");
                    let referent = one(&mut graph, arguments.elements()[0], "referent");
                    let reference = graph.resolve_qualified(reference).unwrap();
                    let Derived::Value(features) = graph.derived(result, "feature") else {
                        panic!("Missing feature")
                    };
                    assert!(
                        features.elements().contains(&reference),
                        "{format:?}: {name}"
                    );
                    assert!(graph.inherited_features(result, true).contains(&reference));
                    assert!(!graph.inheritance_incomplete(result, true));
                    let report = graph.type_feature_report(result);
                    if format == GraphFormat::CanonicalV3 {
                        let checked = report.projections.unwrap_or_else(|issue| {
                            panic!("{name}: {issue:?} after {} steps", report.steps)
                        });
                        assert!(checked.features.contains(&reference));
                    } else {
                        assert_eq!(report.projections, Err(TypeInputIssue::IncompleteProvider));
                    }

                    let quantity = graph
                        .resolve_qualified("Quantities::ScalarQuantityValue")
                        .unwrap();
                    let unit = graph.resolve_qualified("ISQBase::DurationUnit").unwrap();
                    assert!(graph.conforms(referent, quantity));
                    assert!(graph.conforms(reference, unit));
                    let original = graph
                        .resolve_qualified("Quantities::TensorQuantityValue::mRef")
                        .unwrap();
                    let mut seen = HashSet::new();
                    let mut pending = vec![reference];
                    while let Some(feature) = pending.pop() {
                        if seen.insert(feature) {
                            pending.extend(
                                graph
                                    .explicit_specializations(feature)
                                    .into_iter()
                                    .filter_map(|(kind, target)| {
                                        (kind == "Redefinition").then_some(target)
                                    }),
                            );
                        }
                    }
                    assert!(
                        seen.contains(&original),
                        "{format:?}: {name} measurement-reference redefinition"
                    );
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

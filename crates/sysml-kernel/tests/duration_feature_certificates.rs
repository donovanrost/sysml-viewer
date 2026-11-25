use sysmlv2_model::{
    json::{ResolvedModel, TypeInputIssue},
    model::{GraphFormat, Model},
};

#[test]
fn duration_definitions_have_checked_feature_domains() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            for (format, explicit_data_base) in [
                (GraphFormat::CanonicalV3, false),
                (GraphFormat::LegacyV2, false),
                (GraphFormat::LegacyV2, true),
            ] {
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
                let mut text = include_str!(
                    "../../../tests/conformance/fixtures/trigger-arguments-positive.sysml"
                )
                .to_owned();
                if explicit_data_base {
                    for ty in ["CustomDuration", "InheritedDuration", "ISQ::DurationValue"] {
                        text = text
                            .replace(&format!(": {ty};"), &format!(": {ty} :> Base::dataValues;"));
                    }
                }
                assert!(model
                    .add_source("triggers.sysml", &text)
                    .diagnostics
                    .is_empty());
                let mut graph = ResolvedModel::build(&model);
                let package = graph.resolve_qualified("TriggerArguments").unwrap();
                graph.implied_relationships(package);
                for (definition, reference) in [
                    (
                        "TriggerArguments::CustomDuration",
                        "TriggerArguments::CustomDuration::mRef",
                    ),
                    (
                        "TriggerArguments::InheritedDuration",
                        "TriggerArguments::CustomDuration::mRef",
                    ),
                    ("ISQ::DurationValue", "ISQ::DurationValue::mRef"),
                    (
                        "TriggerArguments::delay",
                        "TriggerArguments::CustomDuration::mRef",
                    ),
                    (
                        "TriggerArguments::inheritedDelay",
                        "TriggerArguments::CustomDuration::mRef",
                    ),
                    (
                        "TriggerArguments::ordinaryDelay",
                        "ISQ::DurationValue::mRef",
                    ),
                ] {
                    let owner = graph.resolve_qualified(definition).unwrap();
                    let reference = graph.resolve_qualified(reference).unwrap();
                    let report = graph.type_feature_report(owner);
                    if format == GraphFormat::LegacyV2
                        && !explicit_data_base
                        && graph.element_type(owner) == "AttributeUsage"
                    {
                        let data_values = graph.resolve_qualified("Base::dataValues").unwrap();
                        assert!(!graph.conforms_with_implied(owner, data_values));
                        assert_eq!(report.projections, Err(TypeInputIssue::IncompleteProvider));
                        continue;
                    }
                    let features = report.projections.unwrap_or_else(|issue| {
                        panic!(
                            "{format:?}: {definition}: {issue:?} after {} steps",
                            report.steps
                        )
                    });
                    assert!(features.features.contains(&reference));
                }
                let evaluation_result = graph
                    .resolve_qualified("Performances::Evaluation::result")
                    .unwrap();
                let report = graph.type_feature_report(evaluation_result);
                let features = report.projections.unwrap_or_else(|issue| {
                    panic!(
                        "{format:?}: Evaluation::result: {issue:?} after {} steps",
                        report.steps
                    )
                });
                let anything_self = graph.resolve_qualified("Base::Anything::self").unwrap();
                let things_that = graph.resolve_qualified("Base::things::that").unwrap();
                assert_eq!(features.features.len(), 2);
                assert!(features.features.contains(&anything_self));
                assert!(features.features.contains(&things_that));
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

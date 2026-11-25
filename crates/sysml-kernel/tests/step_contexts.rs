use sysmlv2_model::{
    json::ResolvedModel,
    model::{GraphFormat, Model},
};

#[test]
fn contextual_steps_use_effective_owner_types_with_the_complete_standard_library() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
            let sources: Vec<serde_json::Value> =
                serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/library.json")))
                    .unwrap();
            assert_eq!(sources.len(), 94);
            for source in sources {
                assert!(model
                    .add_library_source(
                        source["name"].as_str().unwrap(),
                        source["text"].as_str().unwrap(),
                    )
                    .diagnostics
                    .is_empty());
            }
            assert!(model
                .add_source(
                    "step-contexts.kerml",
                    "package Contexts {
                behavior B {step enclosed; composite step owned;}
                step outer {step enclosed; composite step owned;}
                struct S {step plain; composite step owned;}
                feature structural:S {composite step owned;}
                feature seed:S;
                feature inherited subsets seed {composite step owned;}
                classifier Narrow specializes S;
                feature narrowed:S,Narrow {composite step owned;}
                step unowned;
            }"
                )
                .diagnostics
                .is_empty());
            let mut graph = ResolvedModel::build(&model);
            let roles = [
                "Performances::performances",
                "Performances::Performance::enclosedPerformances",
                "Performances::Performance::subperformances",
                "Objects::Object::ownedPerformances",
            ]
            .map(|name| graph.resolve_qualified(name).unwrap());
            for (name, expected) in [
                ("B::enclosed", [true, true, false, false]),
                ("B::owned", [true, true, true, false]),
                ("outer::enclosed", [true, true, false, false]),
                ("outer::owned", [true, true, true, false]),
                ("S::plain", [true, false, false, false]),
                ("S::owned", [true, false, false, true]),
                ("structural::owned", [true, false, false, true]),
                ("inherited::owned", [true, false, false, true]),
                ("narrowed::owned", [true, false, false, false]),
                ("unowned", [true, false, false, false]),
            ] {
                let source = graph
                    .resolve_qualified(&format!("Contexts::{name}"))
                    .unwrap();
                assert_eq!(
                    roles.map(|target| graph.conforms_with_implied(source, target)),
                    expected,
                    "{name}"
                );
                assert!(graph.explicit_specializations(source).is_empty());
                let relationships = graph.implied_relationships(source);
                assert_eq!(graph.implied_relationships(source), relationships);
            }
            let structure = graph.resolve_qualified("Contexts::S").unwrap();
            let narrow = graph.resolve_qualified("Contexts::Narrow").unwrap();
            for (name, expected) in [
                ("structural", structure),
                ("inherited", structure),
                ("narrowed", narrow),
            ] {
                let owner = graph
                    .resolve_qualified(&format!("Contexts::{name}"))
                    .unwrap();
                assert_eq!(
                    graph.feature_type_report(owner).types,
                    Ok(vec![expected]),
                    "{name}"
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

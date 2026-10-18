use std::collections::HashSet;
use sysmlv2_model::{
    json::{metaclass_conforms, Derived, ElementRef, ResolvedModel},
    model::{GraphFormat, Model},
};

const SOURCE: &str =
    include_str!("../../../tests/conformance/fixtures/accept-receiver-bindings.sysml");

fn elements(graph: &mut ResolvedModel, owner: ElementRef, property: &str) -> Vec<ElementRef> {
    let Derived::Value(value) = graph.derived(owner, property) else {
        panic!("Missing {property}")
    };
    value.elements()
}

fn value(graph: &mut ResolvedModel, owner: ElementRef, property: &str) -> Option<ElementRef> {
    let Derived::Value(value) = graph.derived(owner, property) else {
        panic!("Missing {property}")
    };
    value.element()
}

fn inside(graph: &mut ResolvedModel, mut element: ElementRef, package: ElementRef) -> bool {
    let mut visited = HashSet::new();
    while visited.insert(element) {
        if element == package {
            return true;
        }
        let Some(owner) = graph.owner(element) else {
            return false;
        };
        element = owner;
    }
    false
}

#[test]
fn trigger_receiver_parameters_and_owned_bindings_are_constructed_together() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
                for with_library in [false, true] {
                    let mut model = Model::with_graph_format(format);
                    if with_library {
                        let sources: Vec<serde_json::Value> = serde_json::from_str(include_str!(
                            concat!(env!("OUT_DIR"), "/library.json")
                        ))
                        .unwrap();
                        for source in sources {
                            model.add_library_source(
                                source["name"].as_str().unwrap(),
                                source["text"].as_str().unwrap(),
                            );
                        }
                    }
                    assert!(model
                        .add_source("receivers.sysml", SOURCE)
                        .diagnostics
                        .is_empty());
                    let mut graph = ResolvedModel::build(&model);
                    let package = graph.resolve_qualified("AcceptReceivers").unwrap();
                    graph.implied_relationships(package);
                    let accepts: Vec<_> = graph
                        .elements_of_metaclass("AcceptActionUsage")
                        .into_iter()
                        .filter(|&e| inside(&mut graph, e, package))
                        .collect();
                    assert_eq!(accepts.len(), 11);
                    let mut triggers = 0;
                    let mut binding_ids = Vec::new();
                    for accept in accepts {
                        let connectors: Vec<_> = elements(&mut graph, accept, "ownedFeature")
                            .into_iter()
                            .filter(|&e| {
                                metaclass_conforms(graph.element_type(e), "BindingConnector")
                            })
                            .collect();
                        let trigger = value(&mut graph, accept, "payloadArgument")
                            .filter(|&e| graph.element_type(e) == "TriggerInvocationExpression");
                        let Some(trigger) = trigger else {
                            assert!(connectors.is_empty());
                            continue;
                        };
                        triggers += 1;
                        let parameters = elements(&mut graph, trigger, "parameter");
                        assert!(
                            parameters.len() >= 2,
                            "trigger receiver missing: format={format:?}, library={with_library}"
                        );
                        let receiver = elements(&mut graph, accept, "parameter")[1];
                        assert_eq!(graph.owner(parameters[1]), Some(trigger));
                        assert_eq!(graph.element_properties(parameters[1])["direction"], "in");
                        assert_eq!(connectors.len(), 1);
                        let binding = connectors[0];
                        assert_eq!(graph.owner(binding), Some(accept));
                        assert_eq!(graph.element_properties(binding)["isImplied"], true);
                        assert_eq!(
                            elements(&mut graph, binding, "relatedFeature"),
                            [receiver, parameters[1]]
                        );
                        let ends = elements(&mut graph, binding, "connectorEnd");
                        assert_eq!(ends.len(), 2);
                        assert_ne!(ends[0], ends[1]);
                        binding_ids.push((accept, graph.element_id(binding)));
                    }
                    assert_eq!(triggers, 9);
                    graph.implied_relationships(package);
                    for (accept, expected) in binding_ids {
                        let connectors: Vec<_> = elements(&mut graph, accept, "ownedFeature")
                            .into_iter()
                            .filter(|&e| {
                                metaclass_conforms(graph.element_type(e), "BindingConnector")
                            })
                            .collect();
                        assert_eq!(connectors.len(), 1);
                        assert_eq!(graph.element_id(connectors[0]), expected);
                    }
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

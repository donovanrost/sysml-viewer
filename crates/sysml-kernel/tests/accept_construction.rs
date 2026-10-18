use sysmlv2_model::{
    json::{Derived, ElementRef, ResolvedModel},
    model::{GraphFormat, Model},
};

const SOURCE: &str = "package AcceptConstruction {
    item def Signal;
    part receiver;
    action def Reception {
        action simple accept data : Signal;
        action directed accept routed : Signal via receiver;
        action absolute accept eventAt at 10;
        action relative accept eventAfter after 5;
        action change accept eventWhen when true via receiver;
    }
    state def Driver {
        state a; state b;
        transition triggered first a accept signal : Signal then b;
    }
}";

fn graph(format: GraphFormat, source: &str) -> ResolvedModel {
    let mut model = Model::with_graph_format(format);
    model.add_source("accepts.sysml", source);
    assert!(
        model.unit(0).diagnostics.is_empty(),
        "{:?}",
        model.unit(0).diagnostics
    );
    ResolvedModel::build(&model)
}

fn derived_elements(graph: &mut ResolvedModel, element: ElementRef, name: &str) -> Vec<ElementRef> {
    let Derived::Value(value) = graph.derived(element, name) else {
        panic!("Missing {name} derivation")
    };
    value.elements()
}

fn derived_element(
    graph: &mut ResolvedModel,
    element: ElementRef,
    name: &str,
) -> Option<ElementRef> {
    let Derived::Value(value) = graph.derived(element, name) else {
        panic!("Missing {name} derivation")
    };
    value.element()
}

#[test]
fn accepts_construct_owned_payload_and_receiver_inputs_even_without_via() {
    for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
        let mut graph = graph(format, SOURCE);
        let accepts = graph.elements_of_metaclass("AcceptActionUsage");
        let counts: Vec<_> = accepts
            .iter()
            .map(|&accept| {
                derived_elements(&mut graph, accept, "input")
                    .into_iter()
                    .filter(|&input| graph.owner(input) == Some(accept))
                    .count()
            })
            .collect();
        assert_eq!(counts, [2, 2, 2, 2, 2, 2]);
        for accept in accepts {
            let inputs: Vec<_> = derived_elements(&mut graph, accept, "input")
                .into_iter()
                .filter(|&input| graph.owner(input) == Some(accept))
                .collect();
            assert_eq!(graph.element_type(inputs[0]), "ReferenceUsage");
            assert_eq!(graph.element_type(inputs[1]), "ReferenceUsage");
            assert_eq!(graph.element_properties(inputs[0])["direction"], "inout");
            assert_eq!(graph.element_name(inputs[1]), None);
            let receiver_member = graph
                .owned_relationships(accept)
                .into_iter()
                .filter(|&member| graph.element_type(member) == "ParameterMembership")
                .nth(1)
                .unwrap();
            let written_via = matches!(graph.element_name(accept), Some("directed" | "change"));
            assert_eq!(
                graph.element_properties(receiver_member)["isImplied"],
                !written_via
            );
            assert_eq!(
                derived_element(&mut graph, accept, "receiverArgument").is_some(),
                written_via
            );
        }
    }
}

#[test]
fn accept_trigger_expressions_are_payload_values_not_sibling_parameters() {
    let mut graph = graph(GraphFormat::CanonicalV3, SOURCE);
    let mut kinds = Vec::new();
    for accept in graph.elements_of_metaclass("AcceptActionUsage") {
        let Some(trigger) = derived_element(&mut graph, accept, "payloadArgument") else {
            continue;
        };
        let payload = derived_element(&mut graph, accept, "payloadParameter").unwrap();
        assert_eq!(graph.element_type(trigger), "TriggerInvocationExpression");
        assert_eq!(graph.owner(trigger), Some(payload));
        assert!(!graph.owned_features(accept).contains(&trigger));
        kinds.push(
            graph.element_properties(trigger)["kind"]
                .as_str()
                .unwrap()
                .to_owned(),
        );
        let argument = graph.owned_features(trigger)[0];
        assert_eq!(graph.element_type(argument), "Feature");
        let values = derived_elements(&mut graph, trigger, "argument");
        assert_eq!(values.len(), 1);
        assert_eq!(graph.owner(values[0]), Some(argument));
        if graph.element_properties(trigger)["kind"] == "when" {
            assert_eq!(graph.element_type(values[0]), "FeatureReferenceExpression");
            let expressions = graph.owned_features(values[0]);
            assert_eq!(expressions.len(), 1);
            assert_eq!(graph.element_type(expressions[0]), "LiteralBoolean");
        } else {
            assert_eq!(graph.element_type(values[0]), "LiteralInteger");
        }
    }
    assert_eq!(kinds, ["at", "after", "when"]);
}

#[test]
fn generated_accept_payload_values_triggers_and_receivers_survive_lifting() {
    let source = SOURCE.replace(
        "action simple",
        "item sample : Signal;
        action boundValue accept boundPayload : Signal = sample via receiver;
        action simple",
    );
    for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
        for source in [
            source.as_str(),
            include_str!("../../../tests/conformance/fixtures/accept-construction.sysml"),
            include_str!("../../../tests/conformance/fixtures/accept-triggers.sysml"),
        ] {
            let mut model = Model::with_graph_format(format);
            model.add_source("accepts.sysml", source);
            assert!(model.unit(0).diagnostics.is_empty());
            let before = sysmlv2_model::json::model_to_compact_json(&model);
            let lifted = sysmlv2_model::lift::from_compact_json(&before).unwrap();
            assert!(lifted.errors.is_empty(), "{:?}", lifted.errors);
            let text = sysmlv2_syntax::print::print_source(&lifted.unit);
            let mut rebuilt = Model::with_graph_format(format);
            rebuilt.add_source("accepts.sysml", &text);
            assert!(rebuilt.unit(0).diagnostics.is_empty(), "{text}");
            assert_eq!(
                sysmlv2_model::json::model_to_compact_json(&rebuilt),
                before,
                "{text}"
            );
        }
    }
}

#[test]
fn state_subactions_and_transition_triggers_keep_their_own_payload_receiver_pairs() {
    let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
    for (name, source) in [
        (
            "ordinary.sysml",
            include_str!("../../../tests/conformance/fixtures/accept-construction.sysml"),
        ),
        (
            "triggers.sysml",
            include_str!("../../../tests/conformance/fixtures/accept-triggers.sysml"),
        ),
    ] {
        model.add_source(name, source);
    }
    assert!(model.units().iter().all(|u| u.diagnostics.is_empty()));
    let mut graph = ResolvedModel::build(&model);
    let accepts = graph.elements_of_metaclass("AcceptActionUsage");
    assert_eq!(accepts.len(), 14);
    for accept in accepts {
        let inputs: Vec<_> = derived_elements(&mut graph, accept, "input")
            .into_iter()
            .filter(|&p| graph.owner(p) == Some(accept))
            .collect();
        let expected = if graph.element_name(accept) == Some("bodyParameters") {
            3
        } else {
            2
        };
        assert_eq!(inputs.len(), expected);
        assert_eq!(graph.element_properties(inputs[0])["direction"], "inout");
        assert_eq!(graph.element_properties(inputs[1])["direction"], "in");
        assert_ne!(inputs[0], inputs[1]);
        if let Some(value) = derived_element(&mut graph, accept, "payloadArgument") {
            assert_eq!(graph.owner(value), Some(inputs[0]));
        }
    }
}

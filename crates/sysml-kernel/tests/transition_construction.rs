use sysmlv2_model::{
    json::{Derived, ElementRef, ResolvedModel},
    model::{GraphFormat, Model},
};

const SOURCE: &str = "package TransitionConstruction {
    item def Signal;
    state def Driver {
        state a; state b;
        transition plain first a then b;
        transition triggered first a accept payload : Signal then b;
    }
}";

fn owned_inputs(graph: &mut ResolvedModel, owner: ElementRef) -> Vec<ElementRef> {
    let Derived::Value(input) = graph.derived(owner, "input") else {
        panic!("Missing input derivation")
    };
    input
        .elements()
        .into_iter()
        .filter(|&input| graph.owner(input) == Some(owner))
        .collect()
}

#[test]
fn transitions_construct_source_and_payload_input_slots_in_order() {
    let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
    model.add_source("transitions.sysml", SOURCE);
    assert!(model.unit(0).diagnostics.is_empty());
    let mut graph = ResolvedModel::build(&model);
    let transitions = graph.elements_of_metaclass("TransitionUsage");
    let counts: Vec<_> = transitions
        .iter()
        .map(|&transition| {
            (
                graph.element_name(transition).unwrap().to_owned(),
                owned_inputs(&mut graph, transition).len(),
            )
        })
        .collect();
    assert_eq!(counts, [("plain".into(), 1), ("triggered".into(), 2)]);
    for transition in transitions {
        let inputs = owned_inputs(&mut graph, transition);
        assert_eq!(graph.element_name(inputs[0]), None);
        assert_eq!(graph.element_type(inputs[0]), "ReferenceUsage");
        if inputs.len() == 2 {
            assert_eq!(graph.element_name(inputs[1]), Some("payload"));
        }
    }
}

#[test]
fn shorthand_guards_effects_and_body_parameters_keep_the_initial_slots() {
    let source = include_str!("../../../tests/conformance/fixtures/transition-construction.sysml");
    for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
        let mut model = Model::with_graph_format(format);
        model.add_source("transitions.sysml", source);
        assert!(
            model.unit(0).diagnostics.is_empty(),
            "{:?}",
            model.unit(0).diagnostics
        );
        let mut graph = ResolvedModel::build(&model);
        let transitions = graph.elements_of_metaclass("TransitionUsage");
        assert_eq!(transitions.len(), 9);
        let mut counts = Vec::new();
        for transition in transitions {
            let inputs = owned_inputs(&mut graph, transition);
            counts.push(inputs.len());
            assert_eq!(graph.element_name(inputs[0]), None);
            assert_eq!(graph.element_type(inputs[0]), "ReferenceUsage");
            if graph.element_name(transition) == Some("full") {
                assert_eq!(
                    inputs
                        .iter()
                        .map(|&p| graph.element_name(p))
                        .collect::<Vec<_>>(),
                    [None, Some("data"), Some("extra"), Some("both")]
                );
            }
        }
        assert_eq!(counts, [1, 2, 1, 1, 4, 2, 1, 1, 1]);
    }
}

#[test]
fn public_membership_queries_preserve_the_constructed_parameter_identities() {
    use sysml_kernel::{AnalysisRequest, Engine, Resolution, SourceDocument, ANALYSIS_STACK_BYTES};
    let snapshot = std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(|| {
            Engine::bundled()
                .analyze(
                    AnalysisRequest {
                        revision: 53,
                        documents: vec![SourceDocument {
                            id: "transitions".into(),
                            name: "transitions.sysml".into(),
                            text: include_str!(
                                "../../../tests/conformance/fixtures/transition-construction.sysml"
                            )
                            .into(),
                        }],
                    },
                    || false,
                )
                .unwrap()
        })
        .unwrap()
        .join()
        .unwrap();
    assert!(
        snapshot.diagnostics.is_empty(),
        "{:?}",
        snapshot.diagnostics
    );
    let query = snapshot.queries().unwrap();
    for (name, payload) in [
        ("plain", None),
        ("triggered", Some("payload")),
        ("full", Some("data")),
    ] {
        let owner = query.find(&format!("TransitionConstruction::Driver::{name}"))[0];
        let members: Vec<_> = owner
            .owned_memberships
            .iter()
            .map(|id| query.membership(id).unwrap())
            .filter(|m| m.metaclass == "ParameterMembership")
            .collect();
        assert_eq!(members.len(), if payload.is_some() { 2 } else { 1 });
        for (index, member) in members.iter().enumerate() {
            assert_eq!(member.namespace_id, owner.id);
            let Resolution::Resolved { target_id } = &member.member else {
                panic!()
            };
            let parameter = query.element(target_id).unwrap();
            assert_eq!(parameter.parent_id.as_deref(), Some(owner.id.as_str()));
            assert_eq!(parameter.metaclass, "ReferenceUsage");
            let parameter_name = if index == 0 {
                "transitionLinkSource"
            } else {
                payload.unwrap()
            };
            let expected = format!("TransitionConstruction::Driver::{name}::{parameter_name}");
            assert_eq!(parameter.qualified_name.as_deref(), Some(expected.as_str()));
        }
        if members.len() == 2 {
            assert_ne!(members[0].member, members[1].member);
        }
    }
}

#[test]
fn parameter_origin_survives_formatting_and_distinguishes_action_and_state_bodies() {
    let source = "state def S {
        state a; state b;
        transition t first a then b;
        state c; if true then b;
    }
    state usage { state a; state b; if true then b; }
    exhibit state shown { state a; state b; if true then b; }
    action def A {
        action a; action b;
        first a if true then b;
        action c; if true then b;
        action d; else b;
        succession named first a if true then b;
    }";
    let formatted =
        sysmlv2_syntax::print::format_source(source, sysmlv2_syntax::ast::Dialect::Sysml).unwrap();
    for text in [source, formatted.as_str()] {
        let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
        model.add_source("origins.sysml", text);
        assert!(
            model.unit(0).diagnostics.is_empty(),
            "{:?}",
            model.unit(0).diagnostics
        );
        let mut graph = ResolvedModel::build(&model);
        let mut implied = Vec::new();
        for transition in graph.elements_of_metaclass("TransitionUsage") {
            assert_eq!(owned_inputs(&mut graph, transition).len(), 1);
            let member = graph
                .owned_relationships(transition)
                .into_iter()
                .find(|&r| graph.element_type(r) == "ParameterMembership")
                .unwrap();
            implied.push(
                graph.element_properties(member)["isImplied"]
                    .as_bool()
                    .unwrap(),
            );
        }
        assert_eq!(
            implied,
            [false, false, false, false, true, true, true, true],
            "{text}"
        );
    }
}

#[test]
fn entry_guards_keep_implied_inputs_across_successions_and_formatting() {
    let source = "state def S {
        entry;
        if true then a;
        if false then b;
        then a;
        if true then b;
        state a; if true then b;
        state b;
    }
    state usage {
        entry action initialize;
        if true then a;
        state a; if false then b;
        state b;
    }";
    let formatted =
        sysmlv2_syntax::print::format_source(source, sysmlv2_syntax::ast::Dialect::Sysml).unwrap();
    for text in [source, formatted.as_str()] {
        let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
        model.add_source("entry-origins.sysml", text);
        assert!(model.unit(0).diagnostics.is_empty());
        let mut graph = ResolvedModel::build(&model);
        let mut implied = Vec::new();
        for transition in graph.elements_of_metaclass("TransitionUsage") {
            assert_eq!(owned_inputs(&mut graph, transition).len(), 1);
            let member = graph
                .owned_relationships(transition)
                .into_iter()
                .find(|&r| graph.element_type(r) == "ParameterMembership")
                .unwrap();
            implied.push(
                graph.element_properties(member)["isImplied"]
                    .as_bool()
                    .unwrap(),
            );
        }
        assert_eq!(implied, [true, true, true, false, true, false], "{text}");
    }
}

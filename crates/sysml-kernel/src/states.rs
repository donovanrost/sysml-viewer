//! Project owned state subaction bindings through the shared semantic query.
//! No inherited-slot synthesis or execution behavior is inferred here.
use crate::{StateSubaction, StateSubactionIssue, StateSubactionKind, StateSubactions};
use sysmlv2_model::{
    check::metaclass_conforms,
    json::{ElementRef, ResolvedModel},
};

pub(crate) fn project(
    graph: &mut ResolvedModel,
    state: ElementRef,
    available: bool,
    invalid_syntax: bool,
    mut id: impl FnMut(ElementRef) -> String,
) -> StateSubactions {
    if !metaclass_conforms(graph.element_type(state), "StateDefinition")
        && !metaclass_conforms(graph.element_type(state), "StateUsage")
    {
        return StateSubactions::NotApplicable;
    }
    let unknown = |reason| StateSubactions::Unknown { reason };
    if !available {
        return unknown(StateSubactionIssue::Unavailable);
    }
    if invalid_syntax {
        return unknown(StateSubactionIssue::InvalidSyntax);
    }
    let bindings: Vec<_> = graph
        .owned_relationships(state)
        .into_iter()
        .filter(|m| metaclass_conforms(graph.element_type(*m), "StateSubactionMembership"))
        .collect();
    let selected = graph.state_subactions(state);
    // The compatibility helper omits unavailable children. It cannot certify a
    // complete set unless every actual owned binding is represented, in order.
    if bindings.len() != selected.len() {
        return unknown(StateSubactionIssue::IncompleteMembership);
    }
    let mut actions = Vec::with_capacity(bindings.len());
    let mut seen = std::collections::HashSet::new();
    for (membership, (kind, action)) in bindings.into_iter().zip(selected) {
        let kind = match kind.as_str() {
            "entry" => StateSubactionKind::Entry,
            "do" => StateSubactionKind::Do,
            "exit" => StateSubactionKind::Exit,
            _ => return unknown(StateSubactionIssue::IncompleteMembership),
        };
        if !seen.insert(membership)
            || graph.owner(membership) != Some(state)
            || graph.owner(action) != Some(state)
            || graph.membership_member(membership) != Some(action)
            || !metaclass_conforms(graph.element_type(action), "ActionUsage")
        {
            return unknown(StateSubactionIssue::IncompleteMembership);
        }
        actions.push(StateSubaction {
            kind,
            membership_id: id(membership),
            action_id: id(action),
        });
    }
    StateSubactions::Complete { actions }
}

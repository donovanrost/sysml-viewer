//! Static action/state graph checks. No scheduling or behavioral execution.
use super::{
    expressions,
    facts::{Facts, flag, is},
};
use crate::{
    json::{ElementRef, ResolvedModel},
    model::Model,
};
use sysmlv2_syntax::diag::Diagnostic;

/// StateDefinition::ownedAction and StateUsage::nestedAction both select
/// ActionUsages from ownedFeature. The inverse TransitionUsage source/target
/// associations matter, not the namespace that happens to own the transition.
fn validate_parallel_states(
    r: &mut ResolvedModel,
    model: &Model,
    g: &Facts,
) -> Vec<(usize, Diagnostic)> {
    let states: Vec<_> = (r.b.lib_boundary..r.b.explicit_len())
        .filter(|&e| {
            (is(&r.b, e, "StateDefinition") || is(&r.b, e, "StateUsage"))
                && flag(&r.b, e, "isParallel")
                && !model.is_library_unit(r.b.unit_of_elem(e))
        })
        .collect();
    if states.is_empty() {
        return Vec::new();
    }
    let mut owners = std::collections::HashMap::new();
    for &state in &states {
        for action in r.owned_features(ElementRef(state)) {
            if is(&r.b, action.0, "ActionUsage")
                && g.owner.get(action.0).copied().flatten() == Some(state)
            {
                owners.insert(action.0, state);
            }
        }
    }
    if owners.is_empty() {
        return Vec::new();
    }
    let mut proof = TransitionEndpointProof::new(r);
    let mut violations = std::collections::HashSet::new();
    for transition in 0..r.b.explicit_len() {
        if !is(&r.b, transition, "TransitionUsage") {
            continue;
        }
        // Incidence is an existence witness. A missing opposite endpoint or
        // an unrelated body failure cannot invalidate a known source/target.
        for endpoint in [
            proof.source(r, g, transition),
            proof.target(r, g, transition),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(&owner) = owners.get(&endpoint) {
                violations.insert(owner);
            }
        }
    }
    states
        .into_iter()
        .filter(|e| violations.contains(e))
        .map(|e| {
            (
                r.b.unit_of_elem(e),
                super::rule_error(
                    g.span(&r.b, e),
                    if is(&r.b, e, "StateDefinition") {
                        "validateStateDefinitionParallelSubactions"
                    } else {
                        "validateStateUsageParallelSubactions"
                    },
                    "An action owned by a parallel state has an incoming or outgoing transition",
                ),
            )
        })
        .collect()
}

/// Guard the shared derived source/target queries against incomplete written
/// inputs. In particular a broken chain must not become its last resolved
/// prefix. This proof does not follow an action's own ReferenceSubsetting:
/// Feature::featureTarget selects its last chainingFeature, or itself.
struct TransitionEndpointProof {
    uncertain: std::collections::HashSet<usize>,
    steps: usize,
}
impl TransitionEndpointProof {
    fn new(r: &ResolvedModel) -> Self {
        Self {
            uncertain: r
                .unresolved_references()
                .into_iter()
                .map(|reference| reference.owner.0)
                .collect(),
            steps: 0,
        }
    }

    fn charge(&mut self, count: usize) -> Option<()> {
        self.steps = self.steps.saturating_add(count);
        (self.steps <= crate::eval::MAX_STEPS).then_some(())
    }

    fn node(&mut self, r: &ResolvedModel, g: &Facts, e: usize) -> Option<()> {
        self.charge(1)?;
        // Facts only index the explicit graph. Generated rows and stale or
        // malformed indices cannot supply a checked endpoint through this path.
        (e < r.b.elements.len() && e < g.owner.len()).then_some(())
    }

    fn reference(&mut self, r: &ResolvedModel, g: &Facts, rel: usize, key: &str) -> Option<usize> {
        self.node(r, g, rel)?;
        if self.uncertain.contains(&rel) {
            return None;
        }
        let target = g.target(&r.b, rel, key)?;
        self.node(r, g, target)?;
        Some(target)
    }

    fn derived_element(r: &mut ResolvedModel, e: usize, property: &str) -> Option<usize> {
        match r.derived(ElementRef(e), property) {
            crate::json::Derived::Value(value) => value.element().map(|e| e.0),
            _ => None,
        }
    }

    fn feature_target(
        &mut self,
        r: &mut ResolvedModel,
        g: &Facts,
        feature: usize,
    ) -> Option<usize> {
        self.node(r, g, feature)?;
        if !is(&r.b, feature, "Feature") {
            return None;
        }
        self.charge(r.b.elements[feature].owned_relationships.len())?;
        for rel in r.b.elements[feature].owned_relationships.to_vec() {
            self.node(r, g, rel)?;
            if is(&r.b, rel, "FeatureChaining") {
                let target = self.reference(r, g, rel, "chainingFeature")?;
                if g.owner[rel] != Some(feature)
                    || !is(&r.b, target, "Feature")
                    || target == feature
                {
                    return None;
                }
            }
        }
        let target = Self::derived_element(r, feature, "featureTarget")?;
        self.node(r, g, target)?;
        Some(target)
    }

    fn source(&mut self, r: &mut ResolvedModel, g: &Facts, transition: usize) -> Option<usize> {
        self.steps = 0;
        self.node(r, g, transition)?;
        self.charge(r.b.elements[transition].owned_relationships.len())?;
        for rel in r.b.elements[transition].owned_relationships.to_vec() {
            self.node(r, g, rel)?;
            if !is(&r.b, rel, "Membership") || is(&r.b, rel, "FeatureMembership") {
                continue;
            }
            // Unknown earlier members cannot be skipped in sourceFeature's
            // ordered selection. Later unrelated members cannot veto a witness.
            let member = self.reference(r, g, rel, "memberElement")?;
            if !is(&r.b, member, "Feature") {
                continue;
            }
            let target = self.feature_target(r, g, member)?;
            if is(&r.b, target, "ActionUsage") {
                return (Self::derived_element(r, transition, "source") == Some(target))
                    .then_some(target);
            }
        }
        None
    }

    fn target(&mut self, r: &mut ResolvedModel, g: &Facts, transition: usize) -> Option<usize> {
        let target = self.target_feature(r, g, transition)?;
        (is(&r.b, target, "ActionUsage")
            && Self::derived_element(r, transition, "target") == Some(target))
        .then_some(target)
    }

    fn target_feature(
        &mut self,
        r: &mut ResolvedModel,
        g: &Facts,
        transition: usize,
    ) -> Option<usize> {
        self.steps = 0;
        self.node(r, g, transition)?;
        self.charge(r.b.elements[transition].owned_relationships.len())?;
        let succession = r
            .owned_members(ElementRef(transition))
            .into_iter()
            .find(|m| is(&r.b, m.0, "Succession"))?
            .0;
        self.node(r, g, succession)?;
        self.charge(r.b.elements[succession].owned_relationships.len())?;
        // Match the shared transition derivation's positional target end: the
        // lowering may leave the first (source) end's reference implicit.
        let end = r
            .owned_features(ElementRef(succession))
            .into_iter()
            .filter(|m| flag(&r.b, m.0, "isEnd"))
            .nth(1)?
            .0;
        self.node(r, g, end)?;
        self.charge(r.b.elements[end].owned_relationships.len())?;
        let mut refs = Vec::new();
        for rel in r.b.elements[end].owned_relationships.to_vec() {
            self.node(r, g, rel)?;
            if is(&r.b, rel, "ReferenceSubsetting") {
                if g.owner[rel] != Some(end) {
                    return None;
                }
                refs.push(rel);
            }
        }
        let [rel] = refs.as_slice() else { return None };
        let feature = self.reference(r, g, *rel, "referencedFeature")?;
        self.feature_target(r, g, feature)
    }
}

pub(super) fn validate(
    r: &mut ResolvedModel,
    model: &Model,
    g: &Facts,
) -> Vec<(usize, Diagnostic)> {
    let mut out = validate_parallel_states(r, model, g);
    let mut endpoint_proof = TransitionEndpointProof::new(r);
    let n = r.b.explicit_len();
    let mut incoming = vec![0; n];
    let mut outgoing = vec![0; n];
    let mut edges = Vec::new();
    // A library succession joins library features only, so the counts it
    // would add are never read: the walks start at the library boundary.
    for e in r.b.lib_boundary..n {
        if !is(&r.b, e, "Succession") {
            continue;
        }
        let ends: Vec<_> = g.members[e]
            .iter()
            .copied()
            .filter(|&m| flag(&r.b, m, "isEnd"))
            .collect();
        let [from, to] = ends.as_slice() else {
            continue;
        };
        let a = g.feature_target(&r.b, *from);
        let z = g.feature_target(&r.b, *to);
        let a = if a == *from {
            g.owner[e]
                .filter(|&p| is(&r.b, p, "TransitionUsage"))
                .and_then(|p| {
                    g.relations(&r.b, p, "Membership", "memberElement")
                        .first()
                        .copied()
                })
        } else {
            Some(a)
        };
        if let Some(a) = a {
            if z != *to {
                incoming[z] += 1;
                outgoing[a] += 1;
                edges.push((e, a, z, *from, *to));
            }
        }
    }
    for (edge, a, z, from, to) in edges {
        let unit = r.b.unit_of_elem(edge);
        if model.is_library_unit(unit) {
            continue;
        }
        for (end, required, rule) in [
            (
                to,
                (1.0, 1.0),
                is(&r.b, z, "ControlNode").then_some("validateControlNodeIncomingSuccessions"),
            ),
            (
                from,
                (1.0, 1.0),
                is(&r.b, a, "ControlNode").then_some("validateControlNodeOutgoingSuccessions"),
            ),
            (
                from,
                (0.0, 1.0),
                is(&r.b, z, "MergeNode").then_some("validateMergeNodeIncomingSuccessions"),
            ),
            (
                to,
                (0.0, 1.0),
                is(&r.b, a, "DecisionNode").then_some("validateDecisionNodeOutgoingSuccessions"),
            ),
        ] {
            if let Some(rule) = rule {
                if r.declared_multiplicity(ElementRef(end))
                    .is_some_and(|m| m != required)
                {
                    out.push((
                        unit,
                        super::rule_error(
                            g.span(&r.b, edge),
                            rule,
                            format!(
                                "Succession end must have multiplicity {}..{}",
                                required.0, required.1
                            ),
                        ),
                    ));
                }
            }
        }
    }
    for e in r.b.lib_boundary..n {
        let unit = r.b.unit_of_elem(e);
        if model.is_library_unit(unit) {
            continue;
        }
        let span = g.span(&r.b, e);
        if is(&r.b, e, "StateSubactionMembership") {
            let valid_owner = g.owner[e].is_some_and(|owner| {
                is(&r.b, owner, "StateDefinition") || is(&r.b, owner, "StateUsage")
            });
            if !valid_owner {
                out.push((
                    unit,
                    super::rule_error(
                        span,
                        "validateStateSubactionMembershipOwningType",
                        "A state subaction membership must belong to a state definition or usage",
                    ),
                ));
            }
        }
        if is(&r.b, e, "TransitionFeatureMembership") {
            if !g.owner[e].is_some_and(|owner| is(&r.b, owner, "TransitionUsage")) {
                out.push((
                    unit,
                    super::rule_error(
                        span,
                        "validateTransitionFeatureMembershipOwningType",
                        "A transition feature membership must belong to a transition usage",
                    ),
                ));
            }
            let required = match r.b.elements[e].props.get("kind").and_then(|v| v.as_str()) {
                Some("trigger") => Some((
                    "AcceptActionUsage",
                    "validateTransitionFeatureMembershipTriggerAction",
                )),
                Some("effect") => Some((
                    "ActionUsage",
                    "validateTransitionFeatureMembershipEffectAction",
                )),
                _ => None,
            };
            if let Some((kind, rule)) = required {
                if !r
                    .membership_member(ElementRef(e))
                    .is_some_and(|member| is(&r.b, member.0, kind))
                {
                    out.push((
                        unit,
                        super::rule_error(
                            span,
                            rule,
                            format!("This transition feature membership must own a {kind}"),
                        ),
                    ));
                }
            }
        }
        for (bad, rule) in [
            (
                is(&r.b, e, "ForkNode") && incoming[e] > 1,
                "validateForkNodeIncomingSuccessions",
            ),
            (
                is(&r.b, e, "DecisionNode") && incoming[e] > 1,
                "validateDecisionNodeIncomingSuccessions",
            ),
            (
                is(&r.b, e, "JoinNode") && outgoing[e] > 1,
                "validateJoinNodeOutgoingSuccessions",
            ),
            (
                is(&r.b, e, "MergeNode") && outgoing[e] > 1,
                "validateMergeNodeOutgoingSuccessions",
            ),
        ] {
            if bad {
                out.push((
                    unit,
                    super::rule_error(
                        span,
                        rule,
                        format!(
                            "Control node has too many {} successions",
                            if rule.contains("Incoming") {
                                "incoming"
                            } else {
                                "outgoing"
                            }
                        ),
                    ),
                ));
            }
        }
        if is(&r.b, e, "TransitionUsage") {
            if endpoint_proof
                .target_feature(r, g, e)
                .is_some_and(|target| !is(&r.b, target, "ActionUsage"))
            {
                out.push((
                    unit,
                    super::rule_error(
                        span,
                        "validateTransitionUsageSuccession",
                        "A transition's succession must target an action usage",
                    ),
                ));
            }
            let triggered = match r.derived(ElementRef(e), "triggerAction") {
                crate::json::Derived::Value(value) => !value.elements().is_empty(),
                _ => false,
            };
            if let crate::json::Derived::Value(input) = r.derived(ElementRef(e), "input") {
                let owned_inputs = input
                    .elements()
                    .into_iter()
                    .filter(|&input| r.owner(input) == Some(ElementRef(e)))
                    .count();
                let required = if triggered { 2 } else { 1 };
                if owned_inputs < required {
                    out.push((unit, super::rule_error(span,
                        "validateTransitionUsageParameters",
                        format!("A transition must have at least {required} owned input parameter(s)"))));
                }
            }
            // The rule constrains the derived source, not every membership or
            // the action referenced by a PerformActionUsage source.
            if triggered
                && endpoint_proof
                    .source(r, g, e)
                    .is_some_and(|source| !is(&r.b, source, "StateUsage"))
            {
                out.push((
                    unit,
                    super::rule_error(
                        span,
                        "validateTransitionUsageTriggerActions",
                        "A triggered transition must have a state usage as its source",
                    ),
                ));
            }
        }
        let params: Vec<_> = g.members[e]
            .iter()
            .copied()
            .filter(|&m| {
                r.b.elements[m]
                    .owning_relationship
                    .is_some_and(|rel| is(&r.b, rel, "ParameterMembership"))
            })
            .collect();
        if is(&r.b, e, "AssignmentActionUsage") {
            if let Some((scope, expr)) = params
                .first()
                .and_then(|p| r.b.contract_exprs.get(p))
                .cloned()
            {
                let origin = r.b.set_identity_origin(e);
                let target = expressions::referent(&mut r.b, scope, &expr);
                r.b.identity_origin_unit = origin;
                if let Some(t) = target {
                    let rule = if !is(&r.b, t, "Feature") {
                        Some((
                            "validateAssignmentActionUsageReferent",
                            "An assignment must refer to a feature",
                        ))
                    } else if !super::structural::variable(&r.b, g, t) {
                        Some((
                            "validateAssignmentActionUsageReferentIsTimeVarying",
                            "An assignment referent must be time varying",
                        ))
                    } else {
                        None
                    };
                    if let Some((rule, msg)) = rule {
                        out.push((unit, super::rule_error(expr.span, rule, msg)));
                    }
                }
            }
        }
        if is(&r.b, e, "SendActionUsage") {
            let needs_payload = r.b.elements[e].owning_relationship.is_some_and(|rel| {
                is(&r.b, rel, "StateSubactionMembership")
                    || is(&r.b, rel, "TransitionFeatureMembership")
            });
            if needs_payload
                && params
                    .first()
                    .is_some_and(|p| !r.b.contract_exprs.contains_key(p))
            {
                out.push((unit, super::rule_error(span, "validateSendActionUsagePayloadArgument", "A send used as a state subaction or transition effect must supply a payload")));
            }
            if let Some((scope, expr)) = params
                .get(2)
                .and_then(|p| r.b.contract_exprs.get(p))
                .cloned()
            {
                let origin = r.b.set_identity_origin(e);
                let target = expressions::referent(&mut r.b, scope, &expr);
                r.b.identity_origin_unit = origin;
                if target.is_some_and(|t| g.effective_kind(&r.b, t, "PortUsage")) {
                    out.push((
                        unit,
                        super::rule_warning(
                            expr.span,
                            "validateSendActionUsageReceiver",
                            "Sending through a port should use 'via'",
                        ),
                    ));
                }
            }
        }
        if is(&r.b, e, "TriggerInvocationExpression") {
            let kind = r.b.elements[e]
                .props
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned();
            let Some((scope, expr)) = r.b.contract_exprs.get(&e).cloned() else {
                continue;
            };
            let (required, rule) = match kind.as_str() {
                "after" => (
                    "ISQ::DurationValue",
                    "validateTriggerInvocationActionAfterArgument",
                ),
                "at" => (
                    "Time::TimeInstantValue",
                    "validateTriggerInvocationActionAtArgument",
                ),
                "when" => (
                    "ScalarValues::Boolean",
                    "validateTriggerInvocationActionWhenArgument",
                ),
                _ => continue,
            };
            let origin = r.b.set_identity_origin(e);
            let mut bad = expressions::wrong_type(&mut r.b, g, scope, &expr, required);
            if kind == "after" {
                if let Some(t) = expressions::library_type(&mut r.b, required) {
                    if let (Some(actual), Some(expected)) = (
                        super::dimensions::known(r, scope, &expr),
                        r.quantity_dims_of_type(ElementRef(t)),
                    ) {
                        bad |= actual != expected;
                    }
                }
            }
            r.b.identity_origin_unit = origin;
            if bad {
                out.push((
                    unit,
                    super::rule_error(
                        expr.span,
                        rule,
                        format!("A '{kind}' trigger argument must conform to {required}"),
                    ),
                ));
            }
        }
    }
    for (e, (scope, expr)) in super::user_entries(&r.b, model, r.b.transition_guards.iter()) {
        let unit = r.b.unit_of_elem(e);
        let origin = r.b.set_identity_origin(e);
        let mut bad = expressions::wrong_type(&mut r.b, g, scope, &expr, "ScalarValues::Boolean");
        if let Some(t) = expressions::referent(&mut r.b, scope, &expr) {
            bad |= r
                .declared_multiplicity(ElementRef(t))
                .is_some_and(|m| m != (1.0, 1.0));
        }
        r.b.identity_origin_unit = origin;
        if bad {
            out.push((
                unit,
                super::rule_error(
                    expr.span,
                    "validateTransitionFeatureMembershipGuardExpression",
                    "A transition guard must have a Boolean result with multiplicity 1..1",
                ),
            ));
        }
    }
    out
}

#[cfg(test)]
mod transition_tests {
    use super::*;

    fn graph(source: &str) -> (Model, ResolvedModel, Facts) {
        let mut model = Model::new();
        model.add_source("states.sysml", source);
        assert!(
            model.unit(0).diagnostics.is_empty(),
            "{:?}",
            model.unit(0).diagnostics
        );
        let mut graph = ResolvedModel::build(&model);
        let facts = Facts::new(&mut graph.b);
        (model, graph, facts)
    }

    fn findings(source: &str) -> Vec<(String, String)> {
        let (model, mut graph, facts) = graph(source);
        let first = validate_parallel_states(&mut graph, &model, &facts);
        let second = validate_parallel_states(&mut graph, &model, &facts);
        assert_eq!(first, second, "derived query order must preserve findings");
        first
            .into_iter()
            .map(|(_, d)| {
                (
                    source[d.span.start as usize..d.span.end as usize].into(),
                    d.code.unwrap().into(),
                )
            })
            .collect()
    }

    #[test]
    fn parallel_state_incidence_is_independent_of_transition_ownership() {
        let result = findings(
            "state def Driver {
            action outside;
            state def Outgoing parallel { action a; }
            transition first Outgoing::a then outside;
            state incoming parallel { action a; }
            transition first outside then incoming.a;
            state local parallel { state a; transition first a then a; }
        }",
        );
        assert_eq!(
            result,
            [
                (
                    "Outgoing".into(),
                    "validateStateDefinitionParallelSubactions".into()
                ),
                (
                    "incoming".into(),
                    "validateStateUsageParallelSubactions".into()
                ),
                (
                    "local".into(),
                    "validateStateUsageParallelSubactions".into()
                ),
            ]
        );
    }

    #[test]
    fn plain_successions_aliases_inheritance_and_grandchildren_do_not_count() {
        assert!(findings("state def Driver {
            action outsideA; action outsideB;
            state def Plain parallel { action a; action b; first a then b; }
            state unrelated parallel { state local; transition first outsideA then outsideB; }
            state aliases parallel { alias borrowed for outsideA; transition first borrowed then outsideB; }
            state def Deep parallel { state region { state a; state b; transition first a then b; } }
            state def Serial { state a; state b; transition first a then b; }
            state def Inherited :> Serial parallel;
            state inherited : Serial parallel;
        }").is_empty());
    }

    #[test]
    fn performed_action_identity_and_state_subaction_memberships_are_retained() {
        let result = findings("state def Driver {
            action outside;
            state performed parallel { perform action local references outside; }
            transition first performed.local then outside;
            state subactions parallel { entry action entering; do action running; exit action leaving; }
            transition first subactions.entering then outside;
            transition first outside then subactions.leaving;
        }");
        assert_eq!(
            result.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
            ["performed", "subactions"]
        );
    }

    #[test]
    fn unknown_chain_links_cannot_supply_endpoints_but_the_other_end_is_independent() {
        let result = findings(
            "state def Driver {
            action outside;
            state sourceBroken parallel { action a; }
            transition first sourceBroken.a.missing then outside;
            state targetBroken parallel { action a; }
            transition first outside then targetBroken.a.missing;
            state middleBroken parallel { action a; }
            transition first middleBroken.missing.a then outside;
            state rootBroken parallel { action a; }
            transition first MissingRoot.rootBroken.a then outside;
            state sourceKnown parallel { action a; }
            transition first sourceKnown.a then Missing;
            state targetKnown parallel { action a; }
            transition first Missing then targetKnown.a;
            state def ValidPeer parallel { state local; }
        }",
        );
        assert_eq!(
            result.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
            ["sourceKnown", "targetKnown"]
        );
    }

    #[test]
    fn ambiguous_source_and_target_do_not_become_arbitrary_endpoints() {
        assert!(
            findings(
                "state def Driver {
            state def Left parallel { action collision; }
            state def Right parallel { action collision; }
            state ambiguous {
                private import Left::collision;
                private import Right::collision;
                transition first collision then collision;
            }
        }"
            )
            .is_empty()
        );
    }

    #[test]
    fn target_kind_does_not_follow_a_reference_usage_or_invent_a_missing_target() {
        let source = "state def Driver {
            action outside;
            attribute scalar;
            ref borrowed references outside;
            perform action local references outside;
            transition toScalar first outside then scalar;
            transition toReference first outside then borrowed;
            transition toPerformed first outside then local;
            transition toMissing first outside then local.missing;
        }";
        let (model, mut graph, facts) = graph(source);
        let names: Vec<_> = validate(&mut graph, &model, &facts)
            .into_iter()
            .filter(|(_, d)| d.code == Some("validateTransitionUsageSuccession"))
            .map(|(_, d)| &source[d.span.start as usize..d.span.end as usize])
            .collect();
        assert_eq!(names, ["toScalar", "toReference"]);
    }

    #[test]
    fn generated_indices_and_exhausted_work_cannot_supply_endpoint_proof() {
        let (_, mut graph, facts) = graph("state def Driver { action a; }");
        let added = graph.b.elements.len();
        graph.b.elements.push(graph.b.elements[0].clone());
        let mut proof = TransitionEndpointProof::new(&graph);
        assert_eq!(proof.feature_target(&mut graph, &facts, added), None);
        proof.steps = crate::eval::MAX_STEPS;
        assert_eq!(proof.node(&graph, &facts, 0), None);
        assert_eq!(proof.charge(usize::MAX), None);
    }
    #[test]
    fn triggered_sources_use_the_first_action_target_and_preserve_perform_identity() {
        let source = "package P { item def Signal; state def Driver {
            state a; state b; action ordinary;
            state region { state nested; perform action work references nested; }
            perform action invoked references a;
            transition performed first invoked accept payload : Signal then b;
            transition performChain first region.work accept payload : Signal then b;
            transition valid first a accept payload : Signal then b { alias unrelated for ordinary; }
            transition chained first region.nested accept payload : Signal then b;
            transition unknown first Missing accept payload : Signal then b;
            transition broken first region.nested.missing accept payload : Signal then b;
            transition invalid first ordinary accept payload : Signal then b;
            transition untriggered first ordinary then b;
        } }";
        let (model, mut graph, facts) = graph(source);
        let findings = validate(&mut graph, &model, &facts);
        let names: Vec<_> = findings
            .iter()
            .filter(|(_, d)| d.code == Some("validateTransitionUsageTriggerActions"))
            .map(|(_, d)| &source[d.span.start as usize..d.span.end as usize])
            .collect();
        assert_eq!(names, ["performed", "performChain", "invalid"]);
    }

    fn codes_after(source: &str, mutate: impl FnOnce(&mut ResolvedModel)) -> Vec<&'static str> {
        let (model, mut graph, _) = graph(source);
        mutate(&mut graph);
        let facts = Facts::new(&mut graph.b);
        validate(&mut graph, &model, &facts)
            .into_iter()
            .filter_map(|(_, d)| d.code)
            .collect()
    }

    #[test]
    fn transition_input_validation_counts_owned_in_and_inout_but_not_output_or_inherited_inputs() {
        let source = "action def Base { in inherited; }
            item def Signal;
            state def Driver { state a; state b;
                transition plain : Base first a then b;
                transition triggered first a accept payload : Signal then b;
            }";
        assert!(codes_after(source, |_| {}).is_empty());
        let failures = codes_after(source, |graph| {
            for transition in graph.elements_of_metaclass("TransitionUsage") {
                let input = graph
                    .owned_features(transition)
                    .into_iter()
                    .find(|&p| graph.element_type(p) == "ReferenceUsage")
                    .unwrap();
                graph.b.elements[input.0]
                    .props
                    .insert("direction", serde_json::json!("out"));
            }
        });
        assert_eq!(
            failures,
            [
                "validateTransitionUsageParameters",
                "validateTransitionUsageParameters"
            ]
        );
        assert!(
            codes_after(source, |graph| {
                for transition in graph.elements_of_metaclass("TransitionUsage") {
                    for input in graph.owned_features(transition) {
                        if graph.element_type(input) == "ReferenceUsage" {
                            graph.b.elements[input.0]
                                .props
                                .insert("direction", serde_json::json!("inout"));
                        }
                    }
                }
            })
            .is_empty()
        );
    }

    #[test]
    fn state_subaction_membership_owner_accepts_state_subtypes_and_rejects_other_namespaces() {
        let source = "state def S { entry; do; exit; }
            state s { entry; } exhibit state shown { exit; }";
        assert!(codes_after(source, |_| {}).is_empty());
        let failures = codes_after(source, |graph| {
            for state in graph.elements_of_metaclass("StateDefinition") {
                graph.b.elements[state.0].ty = "PartDefinition";
            }
        });
        assert_eq!(failures, ["validateStateSubactionMembershipOwningType"; 3]);
    }

    #[test]
    fn transition_membership_ownership_and_feature_kinds_are_independent_checks() {
        let source = "item def Signal;
            state def Driver { state a; state b;
                transition t first a accept payload : Signal do action effect then b;
            }";
        assert!(codes_after(source, |_| {}).is_empty());
        let failures = codes_after(source, |graph| {
            for transition in graph.elements_of_metaclass("TransitionUsage") {
                graph.b.elements[transition.0].ty = "ActionUsage";
            }
        });
        assert_eq!(
            failures,
            ["validateTransitionFeatureMembershipOwningType"; 2]
        );
        let failures = codes_after(source, |graph| {
            let members = graph.elements_of_metaclass("TransitionFeatureMembership");
            for member in members {
                let child = graph.membership_member(member).unwrap();
                graph.b.elements[child.0].ty = match graph.b.elements[member.0]
                    .props
                    .get("kind")
                    .and_then(|x| x.as_str())
                {
                    Some("trigger") => "PerformActionUsage",
                    Some("effect") => "ReferenceUsage",
                    _ => unreachable!(),
                };
            }
        });
        assert_eq!(
            failures,
            [
                "validateTransitionFeatureMembershipTriggerAction",
                "validateTransitionFeatureMembershipEffectAction"
            ]
        );
    }
}

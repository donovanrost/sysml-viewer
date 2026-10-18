//! Accept/trigger receiver construction over a checked, unchanged source graph.
//!
//! Plans admit reciprocal local parameter/value ownership before allocating any
//! semantic nodes. The shared publisher installs a receiver and its binding as
//! one group; source rows, values and identities remain unchanged. This does not
//! qualify trigger argument types, callable defaults or complete library ancestry.
use super::{
    Builder, Elem, membership_evidence as membership,
    owned_results::{BindingMembership, stage_binding},
    semantic_ownership::{self, SemanticOwnership},
    structural_index::StoredStructure,
};
use crate::{metaclass::conforms, properties::Properties};
use serde_json::json;
use std::collections::HashSet;
use uuid::Uuid;

pub(super) struct Plan {
    accept: usize,
    trigger: usize,
    receiver: usize,
    invocation_receiver: Option<usize>,
    parameter_ids: Option<[Uuid; 2]>,
    binding_ids: [Uuid; 8],
    featuring_ids: Vec<Uuid>,
}
impl Plan {
    pub(super) fn ids(&self) -> impl Iterator<Item = Uuid> + '_ {
        self.parameter_ids
            .into_iter()
            .flatten()
            .chain(self.binding_ids)
            .chain(self.featuring_ids.iter().copied())
    }
    pub(super) fn row_count(&self) -> usize {
        self.ids().count()
    }
}
fn charge(steps: &mut usize, n: usize) -> Option<()> {
    *steps = steps.saturating_add(n);
    (*steps <= crate::eval::MAX_STEPS).then_some(())
}
fn id(owner: Uuid, role: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        format!("{owner}/implied/{role}").as_bytes(),
    )
}
fn relationships(
    b: &Builder,
    raw: &StoredStructure,
    owner: usize,
    steps: &mut usize,
) -> Option<Vec<usize>> {
    if !raw.membership_domains(b, steps)?.owner_complete(owner) {
        return None;
    }
    let rows = semantic_ownership::owned_relationships(b, owner)?;
    charge(steps, rows.len())?;
    let rows: Vec<_> = rows.iter().collect();
    let mut seen = HashSet::new();
    for &rel in &rows {
        if !seen.insert(rel)
            || semantic_ownership::checked_relationship_carrier(b, raw, rel, steps)? != Some(owner)
        {
            return None;
        }
        // Conjugated directions require a separate contextual proof.
        if conforms(b.elements[rel].ty, "Conjugation") {
            return None;
        }
    }
    Some(rows)
}
fn features(
    b: &Builder,
    raw: &StoredStructure,
    owner: usize,
    steps: &mut usize,
) -> Option<Vec<usize>> {
    let mut result = Vec::new();
    for rel in relationships(b, raw, owner, steps)? {
        if conforms(b.elements[rel].ty, "FeatureMembership") {
            let feature = membership::member(b, raw, owner, rel, steps)?;
            if !conforms(b.elements[feature].ty, "Feature") {
                return None;
            }
            result.push(feature);
        }
    }
    Some(result)
}
fn parameters(
    b: &Builder,
    raw: &StoredStructure,
    owner: usize,
    steps: &mut usize,
) -> Option<Vec<usize>> {
    let mut result = Vec::new();
    for feature in features(b, raw, owner, steps)? {
        if membership::parameter_direction(b, raw, feature, steps)?.is_some() {
            result.push(feature);
        }
    }
    Some(result)
}
fn input(b: &Builder, raw: &StoredStructure, feature: usize, steps: &mut usize) -> Option<()> {
    matches!(
        membership::parameter_direction(b, raw, feature, steps)?,
        Some("in" | "inout")
    )
    .then_some(())
}
// An existing connector satisfies this obligation by exact endpoint identities,
// not spelling. Malformed owned connectors do not certify absence of a binding.
fn has_binding(
    b: &mut Builder,
    raw: &StoredStructure,
    accept: usize,
    first: usize,
    second: Option<usize>,
    steps: &mut usize,
) -> Option<bool> {
    let mut found = false;
    for binding in features(b, raw, accept, steps)? {
        if !conforms(b.elements[binding].ty, "BindingConnector") {
            continue;
        }
        let mut targets = Vec::new();
        for rel in relationships(b, raw, binding, steps)? {
            if !conforms(b.elements[rel].ty, "EndFeatureMembership") {
                continue;
            }
            let end = membership::member(b, raw, binding, rel, steps)?;
            if !conforms(b.elements[end].ty, "Feature") {
                return None;
            }
            let mut target = None;
            for reference in relationships(b, raw, end, steps)? {
                if !conforms(b.elements[reference].ty, "ReferenceSubsetting") {
                    continue;
                }
                if target.is_some() {
                    return None;
                }
                target = Some(super::type_relations::endpoint(
                    b,
                    end,
                    reference,
                    &["subsettingFeature", "specific"],
                    &["referencedFeature", "subsettedFeature", "general"],
                    "Feature",
                    steps,
                )?);
            }
            targets.push(target?);
        }
        if targets.len() != 2 {
            return None;
        }
        found |= second.is_some_and(|second| targets.contains(&first) && targets.contains(&second));
    }
    Some(found)
}
fn candidate(
    b: &mut Builder,
    raw: &StoredStructure,
    accept: usize,
    steps: &mut usize,
) -> Option<Plan> {
    let accepts = parameters(b, raw, accept, steps)?;
    let (&payload, &receiver) = (accepts.first()?, accepts.get(1)?);
    input(b, raw, payload, steps)?;
    input(b, raw, receiver, steps)?;
    let (_, trigger) = membership::valuation(b, raw, payload, steps)??;
    if b.elements[trigger].ty != "TriggerInvocationExpression"
        || !matches!(
            b.elements[trigger]
                .props
                .get("kind")
                .and_then(|v| v.as_str()),
            Some("at" | "after" | "when")
        )
    {
        return None;
    }
    let parameters = parameters(b, raw, trigger, steps)?;
    let first = *parameters.first()?;
    input(b, raw, first, steps)?;
    // The grammar argument must own its value; a missing or inconsistent value
    // is not repaired by inventing the unrelated receiver argument.
    membership::valuation(b, raw, first, steps)??;
    let invocation_receiver = parameters.get(1).copied();
    if let Some(second) = invocation_receiver {
        input(b, raw, second, steps)?;
    }
    // Audit existing bindings even when a receiver must be synthesized. A
    // malformed connector is not proof that the required connection is absent.
    if has_binding(b, raw, accept, receiver, invocation_receiver, steps)? {
        return None;
    }
    let trigger_id = b.elements[trigger].id;
    let parameter_ids = invocation_receiver.is_none().then(|| {
        [
            id(trigger_id, "receiverMembership"),
            id(trigger_id, "receiver"),
        ]
    });
    let second_id = invocation_receiver
        .map(|e| b.elements[e].id)
        .unwrap_or_else(|| parameter_ids.unwrap()[1]);
    let accept_id = b.elements[accept].id;
    let binding_id = id(accept_id, &format!("triggerReceiverBinding/{trigger_id}"));
    let end0 = id(binding_id, "end/0");
    let end1 = id(binding_id, "end/1");
    let binding_ids = [
        id(
            accept_id,
            &format!("triggerReceiverMembership/{trigger_id}"),
        ),
        binding_id,
        id(binding_id, "endMembership/0"),
        end0,
        id(
            end0,
            &format!("ReferenceSubsetting/{}", b.elements[receiver].id),
        ),
        id(binding_id, "endMembership/1"),
        end1,
        id(end1, &format!("ReferenceSubsetting/{second_id}")),
    ];
    let mut featuring_ids = vec![
        id(binding_id, &format!("TypeFeaturing/{accept_id}")),
        id(end0, &format!("TypeFeaturing/{binding_id}")),
        id(end1, &format!("TypeFeaturing/{binding_id}")),
    ];
    if parameter_ids.is_some() {
        featuring_ids.push(id(second_id, &format!("TypeFeaturing/{trigger_id}")));
    }
    Some(Plan {
        accept,
        trigger,
        receiver,
        invocation_receiver,
        parameter_ids,
        binding_ids,
        featuring_ids,
    })
}
/// None is a work-limited attempt: publish none of this family. An inadmissible
/// accept is skipped independently; absence from the plan is never completeness.
pub(super) fn plan(
    b: &mut Builder,
    reserved: &HashSet<Uuid>,
    steps: &mut usize,
) -> Option<Vec<Plan>> {
    let raw = StoredStructure::get(b, steps)?;
    if !raw.ids_unique {
        return Some(Vec::new());
    }
    let end = b.explicit_len();
    charge(steps, end)?;
    let mut allocated = HashSet::new();
    let mut plans = Vec::new();
    for accept in 0..end {
        if !conforms(b.elements[accept].ty, "AcceptActionUsage") {
            continue;
        }
        let Some(plan) = candidate(b, &raw, accept, steps) else {
            if *steps > crate::eval::MAX_STEPS {
                return None;
            }
            continue;
        };
        charge(steps, plan.row_count())?;
        let ids: HashSet<_> = plan.ids().collect();
        if ids.len() != plan.row_count()
            || ids.iter().any(|uuid| {
                raw.element_for_uuid(b, *uuid).is_some()
                    || reserved.contains(uuid)
                    || allocated.contains(uuid)
            })
        {
            continue;
        }
        allocated.extend(ids);
        plans.push(plan);
    }
    Some(plans)
}
fn node(
    ty: &'static str,
    id: Uuid,
    owning_relationship: Option<usize>,
    children: Vec<usize>,
    owned_relationships: Vec<usize>,
    props: Properties,
) -> Elem {
    Elem {
        ty,
        id,
        path: String::new(),
        path_parent: None,
        owning_relationship,
        children: children.into(),
        owned_relationships: owned_relationships.into(),
        props,
    }
}
/// A prevalidated group is appended only to temporary rows and a private ownership
/// view. The caller publishes both together after the complete transaction.
pub(super) fn stage(
    b: &Builder,
    view: &mut SemanticOwnership,
    plan: Plan,
    rows: &mut Vec<Elem>,
) -> Vec<(Uuid, Uuid)> {
    let mut receiver = plan.invocation_receiver;
    if let Some(ids) = plan.parameter_ids {
        let member = b.elements.len() + rows.len();
        let feature = member + 1;
        let mut props = Properties::new();
        props.insert("isImplied", json!(true));
        props.insert("visibility", json!("private"));
        props.insert(
            "owningRelatedElement",
            json!({"@id":b.elements[plan.trigger].id.to_string()}),
        );
        rows.push(node(
            "ParameterMembership",
            ids[0],
            None,
            vec![feature],
            vec![],
            props,
        ));
        let mut props = Properties::new();
        props.insert("direction", json!("in"));
        props.insert("isComposite", json!(false));
        rows.push(node("Feature", ids[1], Some(member), vec![], vec![], props));
        view.register(member, Some(plan.trigger), plan.accept, true)
            .expect("planned receiver membership");
        view.register(feature, None, plan.accept, false)
            .expect("planned receiver feature");
        receiver = Some(feature);
    }
    let receiver = receiver.unwrap();
    let receiver_id = plan
        .parameter_ids
        .map(|p| p[1])
        .unwrap_or_else(|| b.elements[receiver].id);
    let start = b.elements.len() + rows.len();
    let (binding, first, second) = stage_binding(
        b,
        view,
        plan.accept,
        plan.accept,
        BindingMembership::Feature,
        plan.receiver,
        receiver_id,
        &plan.binding_ids,
        start,
        rows,
    );
    let mut roles = vec![(binding, plan.accept), (first, binding), (second, binding)];
    if plan.parameter_ids.is_some() {
        roles.push((receiver, plan.trigger));
    }
    for ((source, target), id) in roles.into_iter().zip(plan.featuring_ids) {
        let index = b.elements.len() + rows.len();
        let source_id = rows[source - b.elements.len()].id;
        let target_id = if target < b.elements.len() {
            b.elements[target].id
        } else {
            rows[target - b.elements.len()].id
        };
        let mut props = Properties::new();
        props.insert("isImplied", json!(true));
        props.insert("owningRelatedElement", json!({"@id":source_id.to_string()}));
        props.insert("featureOfType", json!({"@id":source_id.to_string()}));
        props.insert("featuringType", json!({"@id":target_id.to_string()}));
        rows[source - b.elements.len()]
            .owned_relationships
            .push(index);
        rows.push(node("TypeFeaturing", id, None, vec![], vec![], props));
        view.register(index, Some(source), plan.accept, false)
            .expect("planned featuring relationship");
    }
    vec![
        (plan.binding_ids[3], b.elements[plan.receiver].id),
        (plan.binding_ids[6], receiver_id),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        json::{Derived, ElementRef, ResolvedModel},
        model::{GraphFormat, Model},
    };

    fn fixture(format: GraphFormat) -> ResolvedModel {
        let mut model = Model::with_graph_format(format);
        model.add_source(
            "receivers.sysml",
            "action a accept p after 1; action b accept q when true; item outsider;",
        );
        assert!(!model.has_errors());
        ResolvedModel::build(&model)
    }
    fn plans(b: &mut Builder) -> Vec<Plan> {
        plan(b, &HashSet::new(), &mut 0).unwrap()
    }
    fn snapshot(b: &Builder, end: usize) -> serde_json::Value {
        json!(
            b.elements
                .iter()
                .take(end)
                .map(|e| json!({
                    "id":e.id, "type":e.ty, "props":e.props.to_json(),
                    "owner":e.owning_relationship,
                    "children":e.children.iter().copied().collect::<Vec<_>>(),
                    "relationships":e.owned_relationships.iter().copied().collect::<Vec<_>>()
                }))
                .collect::<Vec<_>>()
        )
    }
    // Deliberate abstract-model construction exercises shapes textual grammar
    // does not emit, independently of the production binding factory.
    fn owned(b: &mut Builder, owner: usize, membership: &'static str, ty: &'static str) -> usize {
        let m = b.elements.len();
        let child = m + 1;
        let owner_id = b.elements[owner].id;
        let mut props = Properties::new();
        props.insert("owningRelatedElement", json!({"@id":owner_id}));
        b.elements.push(node(
            membership,
            Uuid::new_v5(&owner_id, format!("test/{m}").as_bytes()),
            None,
            vec![child],
            vec![],
            props,
        ));
        b.elements.push(node(
            ty,
            Uuid::new_v5(&owner_id, format!("test/{child}").as_bytes()),
            Some(m),
            vec![],
            vec![],
            Properties::new(),
        ));
        b.elements[owner].owned_relationships.push(m);
        child
    }
    fn receiver(b: &mut Builder, trigger: usize) -> usize {
        let feature = owned(b, trigger, "ParameterMembership", "Feature");
        b.elements[feature].props.insert("direction", json!("in"));
        feature
    }
    fn binding(b: &mut Builder, accept: usize, targets: [usize; 2]) -> usize {
        let connector = owned(b, accept, "FeatureMembership", "BindingConnector");
        for target in targets {
            let end = owned(b, connector, "EndFeatureMembership", "Feature");
            b.elements[end].props.insert("isEnd", json!(true));
            let index = b.elements.len();
            let end_id = b.elements[end].id;
            let mut props = Properties::new();
            props.insert("owningRelatedElement", json!({"@id":end_id}));
            props.insert("subsettingFeature", json!({"@id":end_id}));
            props.insert("referencedFeature", json!({"@id":b.elements[target].id}));
            b.elements.push(node(
                "ReferenceSubsetting",
                Uuid::new_v5(&end_id, b"test/reference"),
                None,
                vec![],
                vec![],
                props,
            ));
            b.elements[end].owned_relationships.push(index);
        }
        connector
    }
    fn assert_only_neighbor(b: &mut Builder, broken: usize) {
        let admitted = plans(b);
        assert_eq!(admitted.len(), 1);
        assert_ne!(admitted[0].accept, broken);
    }

    #[test]
    fn malformed_local_evidence_refuses_the_accept_without_erasing_its_neighbor() {
        for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
            for mutation in 0..8 {
                let mut r = fixture(format);
                let p = plans(&mut r.b).remove(0);
                let raw = StoredStructure::get(&mut r.b, &mut 0).unwrap();
                let payload = parameters(&r.b, &raw, p.accept, &mut 0).unwrap()[0];
                let first = parameters(&r.b, &raw, p.trigger, &mut 0).unwrap()[0];
                let payload_member = r.b.elements[payload].owning_relationship.unwrap();
                let first_value = membership::valuation(&r.b, &raw, first, &mut 0)
                    .unwrap()
                    .unwrap()
                    .0;
                let wrong_id = r.b.elements[p.accept].id;
                match mutation {
                    0 => {
                        r.b.elements[p.trigger]
                            .props
                            .insert("kind", json!("invalid"));
                    }
                    1 => {
                        r.b.elements[p.receiver]
                            .props
                            .insert("direction", json!("out"));
                    }
                    2 => {
                        r.b.elements[payload].owning_relationship = None;
                    }
                    3 => {
                        r.b.elements[payload_member]
                            .props
                            .insert("ownedMemberParameter", json!({"@id":wrong_id}));
                    }
                    4 => {
                        r.b.elements[first_value]
                            .props
                            .insert("value", json!({"@id":wrong_id}));
                    }
                    5 => {
                        r.b.elements[p.trigger].owning_relationship = None;
                    }
                    6 => {
                        r.b.elements[first].props.insert("direction", json!(null));
                    }
                    7 => {
                        r.b.elements[payload_member]
                            .props
                            .insert("source", json!([{ "@id":wrong_id }, { "@id":wrong_id }]));
                    }
                    _ => unreachable!(),
                }
                assert_only_neighbor(&mut r.b, p.accept);
            }
        }
    }

    #[test]
    fn existing_receiver_is_reused_and_matching_bindings_accept_either_end_order() {
        for reversed in [false, true] {
            let mut r = fixture(GraphFormat::CanonicalV3);
            let p = plans(&mut r.b).remove(0);
            let second = receiver(&mut r.b, p.trigger);
            let reuse = plans(&mut r.b).remove(0);
            assert_eq!(reuse.invocation_receiver, Some(second));
            assert!(reuse.parameter_ids.is_none());
            let mut ends = [p.receiver, second];
            if reversed {
                ends.reverse();
            }
            binding(&mut r.b, p.accept, ends);
            assert_only_neighbor(&mut r.b, p.accept);
            r.ensure_implied();
            let Derived::Value(features) = r.derived(ElementRef(p.accept), "ownedFeature") else {
                panic!()
            };
            assert_eq!(
                features
                    .elements()
                    .iter()
                    .filter(|&&e| r.element_type(e) == "BindingConnector")
                    .count(),
                1
            );
        }
    }

    #[test]
    fn unrelated_binding_is_preserved_but_does_not_replace_the_required_connection() {
        let mut r = fixture(GraphFormat::CanonicalV3);
        let p = plans(&mut r.b).remove(0);
        let second = receiver(&mut r.b, p.trigger);
        let outsider = r.resolve_qualified("outsider").unwrap().0;
        let original = binding(&mut r.b, p.accept, [p.receiver, outsider]);
        let before = snapshot(&r.b, r.b.elements.len());
        let count = r.b.elements.len();
        let admitted = plans(&mut r.b);
        assert_eq!(admitted.len(), 2);
        assert_eq!(admitted[0].invocation_receiver, Some(second));
        r.ensure_implied();
        assert_eq!(snapshot(&r.b, count), before);
        let Derived::Value(features) = r.derived(ElementRef(p.accept), "ownedFeature") else {
            panic!()
        };
        let bindings: Vec<_> = features
            .elements()
            .into_iter()
            .filter(|&e| r.element_type(e) == "BindingConnector")
            .collect();
        assert_eq!(bindings.len(), 2);
        assert!(bindings.contains(&ElementRef(original)));
    }

    #[test]
    fn malformed_existing_bindings_are_not_treated_as_absent_even_without_a_receiver() {
        for supplied in [false, true] {
            for mutation in 0..6 {
                let mut r = fixture(GraphFormat::CanonicalV3);
                let p = plans(&mut r.b).remove(0);
                let outsider = r.resolve_qualified("outsider").unwrap().0;
                let target = if supplied {
                    receiver(&mut r.b, p.trigger)
                } else {
                    outsider
                };
                let connector = binding(&mut r.b, p.accept, [p.receiver, target]);
                let end_member = r.b.elements[connector].owned_relationships[0];
                let end = r.b.elements[end_member].children[0];
                let reference = r.b.elements[end].owned_relationships[0];
                let wrong_id = r.b.elements[outsider].id;
                match mutation {
                    0 => {
                        r.b.elements[reference]
                            .props
                            .insert("general", json!({"@id":wrong_id}));
                    }
                    1 => {
                        r.b.elements[reference]
                            .props
                            .insert("target", json!([{ "@id":wrong_id }]));
                    }
                    2 => {
                        r.b.elements[end].owning_relationship = None;
                    }
                    3 => {
                        r.b.elements[end].owned_relationships = Default::default();
                    }
                    4 => {
                        r.b.elements[end_member]
                            .props
                            .insert("owningRelatedElement", json!({"@id":wrong_id}));
                    }
                    5 => {
                        r.b.elements[end].ty = "Class";
                    }
                    _ => unreachable!(),
                }
                assert_only_neighbor(&mut r.b, p.accept);
            }
        }
    }

    #[test]
    fn every_generated_identity_is_reserved_before_any_rows_can_publish() {
        let mut original = fixture(GraphFormat::CanonicalV3);
        let p = plans(&mut original.b).remove(0);
        let outsider = original.resolve_qualified("outsider").unwrap().0;
        for uuid in p.ids() {
            let mut b = original.b.clone();
            let before = snapshot(&b, b.elements.len());
            let admitted = plan(&mut b, &HashSet::from([uuid]), &mut 0).unwrap();
            assert_eq!(admitted.len(), 1);
            assert_ne!(admitted[0].accept, p.accept);
            assert_eq!(snapshot(&b, b.elements.len()), before);
            b.elements[outsider].id = uuid;
            b.id_index = None;
            assert_only_neighbor(&mut b, p.accept);
        }
        original.b.elements[outsider].id = original.b.elements[p.accept].id;
        original.b.id_index = None;
        assert!(plans(&mut original.b).is_empty());
    }

    #[test]
    fn exhausted_planning_discards_earlier_plans_without_mutating_source() {
        let r = fixture(GraphFormat::CanonicalV3);
        let mut measured = r.b.clone();
        let mut used = 0;
        assert_eq!(
            plan(&mut measured, &HashSet::new(), &mut used)
                .unwrap()
                .len(),
            2
        );
        assert!(used > 1 && used < crate::eval::MAX_STEPS);
        let mut limited = r.b.clone();
        let before = snapshot(&limited, limited.elements.len());
        let mut steps = crate::eval::MAX_STEPS - used + 1;
        assert!(plan(&mut limited, &HashSet::new(), &mut steps).is_none());
        assert!(steps > crate::eval::MAX_STEPS);
        assert_eq!(snapshot(&limited, limited.elements.len()), before);
        assert_eq!(plans(&mut limited).len(), 2, "a fresh attempt can succeed");
    }

    #[test]
    fn publication_and_tail_replay_preserve_source_and_generated_identities() {
        for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
            let mut r = fixture(format);
            let p = plans(&mut r.b).remove(0);
            let end = r.b.elements.len();
            let before = snapshot(&r.b, end);
            let _ = r.derived(ElementRef(p.trigger), "parameter");
            r.ensure_implied();
            let after = snapshot(&r.b, r.b.elements.len());
            assert_eq!(snapshot(&r.b, end), before);
            assert!(
                plans(&mut r.b).is_empty(),
                "published bindings must be recognized"
            );
            let Derived::Value(parameters) = r.derived(ElementRef(p.trigger), "parameter") else {
                panic!()
            };
            assert_eq!(parameters.elements().len(), 2);
            r.discard_owned_result_tail();
            r.ensure_implied();
            assert_eq!(snapshot(&r.b, r.b.elements.len()), after);
            let Derived::Value(replayed) = r.derived(ElementRef(p.trigger), "parameter") else {
                panic!()
            };
            assert_eq!(replayed.elements(), parameters.elements());
        }
    }
}

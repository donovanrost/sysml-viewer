//! Contextual Step requirements determined by the owning Type's metaclass.
//!
//! The pinned KerML rules use singular names, while their referenced library
//! declarations are plural. The project interpretation and open authority
//! discrepancy are recorded in docs/evidence/step-context-construction-review.json.
//! These are positive construction requirements, not a complete Step certificate.
//! A composite Step owned by a Feature additionally needs that owner's effective
//! Feature.type projection; written typing alone cannot establish that condition.
use super::{Builder, structural_index::StoredStructure, type_relations::TypeRelations};
use crate::metaclass::conforms;
use std::{collections::HashSet, sync::Arc};

pub(super) const ENCLOSED: &str = "Performances::Performance::enclosedPerformances";
pub(super) const SUBPERFORMANCES: &str = "Performances::Performance::subperformances";
pub(super) const OWNED: &str = "Objects::Object::ownedPerformances";
pub(super) const ROLES: [&str; 3] = [ENCLOSED, SUBPERFORMANCES, OWNED];

/// Families whose complete membership proof can use the common noncomposite
/// Step/Expression requirements. Reference sources additionally require their
/// construction certificate in the membership provider. Invocations remain separate.
pub(super) fn supports_membership_domain(kind: &str) -> bool {
    matches!(
        kind,
        "Step"
            | "Expression"
            | "FeatureReferenceExpression"
            | "BooleanExpression"
            | "LiteralExpression"
            | "LiteralBoolean"
            | "LiteralInteger"
            | "LiteralInfinity"
            | "LiteralRational"
            | "LiteralString"
            | "NullExpression"
    )
}

pub(super) const EXPRESSION_ROLES: [&str; 7] = [
    "Performances::booleanEvaluations",
    "Performances::literalEvaluations",
    "Performances::literalBooleanEvaluations",
    "Performances::literalIntegerEvaluations",
    "Performances::literalRationalEvaluations",
    "Performances::literalStringEvaluations",
    "Performances::nullEvaluations",
];

pub(super) fn typed_candidate(element: &super::Elem) -> bool {
    conforms(element.ty, "Step")
        && element.props.get("isComposite").and_then(|v| v.as_bool()) == Some(true)
}

pub(super) struct TypedRequirements {
    pub(super) edges: Vec<(usize, usize)>,
    pub(super) incomplete: HashSet<usize>,
    /// True and false applicability facts both require the final-write audit.
    pub(super) has_stable_predicate: bool,
}

/// Prepare only requirements whose complete owner typing is independent of
/// every possible specialization writer in the enclosing atomic transaction.
/// Coupled predicates remain unknown; they need a mutually stable graph proof.
pub(super) fn typed_requirements(
    b: &mut Builder,
    changing: Arc<HashSet<usize>>,
    raw: &Arc<StoredStructure>,
    steps: &mut usize,
) -> Option<TypedRequirements> {
    let mut relations = TypeRelations::with_structure(Arc::clone(raw));
    let mut candidates = Vec::new();
    let mut owners = Vec::new();
    let mut seen = HashSet::new();
    let mut incomplete = HashSet::new();
    *steps = steps.saturating_add(b.explicit_len());
    if *steps > crate::eval::MAX_STEPS {
        return None;
    }
    for source in 0..b.explicit_len() {
        if !typed_candidate(&b.elements[source]) {
            continue;
        }
        let owner = match relations.owning_type(b, source, steps) {
            Some(Some(owner)) => owner,
            Some(None) => continue,
            None => {
                incomplete.insert(source);
                continue;
            }
        };
        if !conforms(b.elements[owner].ty, "Feature") {
            continue;
        }
        let membership = b.elements[source].owning_relationship?;
        if !raw.membership_domains(b, steps)?.owner_complete(owner)
            || super::membership_evidence::member(b, raw, owner, membership, steps) != Some(source)
        {
            incomplete.insert(source);
            continue;
        }
        candidates.push((source, owner));
        if seen.insert(owner) {
            owners.push(owner);
        }
    }
    let mut result = TypedRequirements {
        edges: Vec::new(),
        incomplete,
        has_stable_predicate: false,
    };
    if owners.is_empty() {
        return Some(result);
    }
    let projected =
        super::feature_type_projection::structural_owners_for_planning(b, &owners, changing, steps);
    if *steps > crate::eval::MAX_STEPS {
        return None;
    }
    let projected = projected.unwrap_or_default();
    let target = relations.library_role(b, OWNED, steps);
    if *steps > crate::eval::MAX_STEPS {
        return None;
    }
    for (source, owner) in candidates {
        *steps = steps.saturating_add(1);
        match projected.get(&owner) {
            Some(true) => {
                result.has_stable_predicate = true;
                if let Some(target) = target {
                    result.edges.push((source, target));
                } else {
                    result.incomplete.insert(source);
                }
            }
            Some(false) => result.has_stable_predicate = true,
            None => {
                result.incomplete.insert(source);
            }
        }
    }
    (*steps <= crate::eval::MAX_STEPS).then_some(result)
}

/// Positive requirements independent of effective owner typing. An unavailable
/// ownership proof contributes no guessed edges. The caller still retains all
/// independently established kind requirements and propagates budget exhaustion.
pub(super) fn metaclass_requirements(
    b: &mut Builder,
    source: usize,
    raw: &StoredStructure,
    relations: &mut TypeRelations,
    steps: &mut usize,
) -> Option<Vec<&'static str>> {
    *steps = steps.saturating_add(1);
    if *steps > crate::eval::MAX_STEPS
        || source >= b.explicit_len()
        || !conforms(b.elements.get(source)?.ty, "Step")
        || raw.incomplete()
    {
        return None;
    }
    let Some(owner) = relations.owning_type(b, source, steps)? else {
        return Some(Vec::new());
    };
    let membership = b.elements[source].owning_relationship?;
    let domains = raw.membership_domains(b, steps)?;
    if !domains.owner_complete(owner)
        || super::membership_evidence::member(b, raw, owner, membership, steps) != Some(source)
    {
        return None;
    }
    let owner_kind = b.elements[owner].ty;
    let behavioral = conforms(owner_kind, "Behavior") || conforms(owner_kind, "Step");
    let composite = b.elements[source]
        .props
        .get("isComposite")
        .and_then(|v| v.as_bool());
    let mut roles = Vec::new();
    if behavioral {
        roles.push(ENCLOSED);
        // Invalid flags cannot supply a composite witness, but they do not
        // erase the independent enclosure requirement established above.
        if composite == Some(true) {
            roles.push(SUBPERFORMANCES);
        }
    }
    if composite == Some(true) && conforms(owner_kind, "Structure") {
        roles.push(OWNED);
    }
    Some(roles)
}

/// Complete contextual requirements for the noncomposite reference-expression
/// source admitted by the generated-result recipe. Directed parameters and
/// result-expression memberships have separate positional obligations; they
/// cannot enter this domain merely because they have an owning Type.
pub(super) fn reference_requirements(
    b: &mut Builder,
    source: usize,
    relations: &mut TypeRelations,
    steps: &mut usize,
) -> Option<Vec<&'static str>> {
    let raw = StoredStructure::for_query(b, steps)?;
    let element = b.elements.get(source)?;
    if element.ty != "FeatureReferenceExpression"
        || element
            .props
            .get("isComposite")
            .is_some_and(|v| v.as_bool() != Some(false))
        || element.props.get("direction").is_some_and(|v| !v.is_null())
    {
        return None;
    }
    // This certificate covers constructed, owned expression sources. A missing
    // backlink is not evidence that no other membership claims the source.
    // Root abstract expressions need a separate complete inverse-absence proof.
    let membership = element.owning_relationship?;
    if relations.owning_type(b, source, steps)?.is_some()
        && b.elements.get(membership)?.ty != "FeatureMembership"
    {
        return None;
    }
    metaclass_requirements(b, source, &raw, relations, steps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        json::{ElementRef, ResolvedModel},
        model::{GraphFormat, Model},
    };
    use serde_json::json;

    const LIB: &str = "standard library package Base {classifier Anything; feature things:Anything;}
standard library package Occurrences {class Occurrence specializes Base::Anything; feature occurrences:Occurrence subsets Base::things;}
standard library package Performances {
 behavior Performance specializes Occurrences::Occurrence {
  step enclosedPerformances:Performance subsets performances;
  composite step subperformances:Performance subsets enclosedPerformances;
 }
 step performances:Performance subsets Occurrences::occurrences;
 function Evaluation specializes Performance {return result;}
 expr evaluations:Evaluation subsets performances;
}
standard library package Objects {struct Object specializes Occurrences::Occurrence {
 composite step ownedPerformances:Performances::Performance subsets Performances::performances;
} feature objects:Object subsets Occurrences::occurrences;}";
    const SOURCE: &str = "behavior B {step enclosed; composite step owned;}
step outer {step enclosed; composite step owned;}
struct S {step plain; composite step owned;}
function F {step enclosed;}
predicate P {step enclosed;}
step unowned;";

    fn fixture(format: GraphFormat) -> ResolvedModel {
        let mut m = Model::with_graph_format(format);
        assert!(
            m.add_library_source("step-roles.kerml", LIB)
                .diagnostics
                .is_empty()
        );
        assert!(m.add_source("steps.kerml", SOURCE).diagnostics.is_empty());
        ResolvedModel::build(&m)
    }
    const TYPED_SOURCE: &str = "struct S specializes Objects::Object;
classifier Narrow specializes S;
feature structural:S subsets Objects::objects {composite step owned; step plain; composite step another;}
feature seed:S subsets Objects::objects;
feature inherited subsets seed {composite step owned;}
feature narrowed:S,Narrow subsets Objects::objects {composite step owned;}";

    fn typed_fixture(format: GraphFormat, library: &str, source: &str) -> ResolvedModel {
        let mut m = Model::with_graph_format(format);
        for (name, text, library) in [
            ("step-roles.kerml", library, true),
            ("typed.kerml", source, false),
        ] {
            let result = if library {
                m.add_library_source(name, text)
            } else {
                m.add_source(name, text)
            };
            assert!(
                result.diagnostics.is_empty(),
                "{name}: {:?}",
                result.diagnostics
            );
        }
        ResolvedModel::build(&m)
    }

    #[test]
    fn typed_step_uses_effective_owner_types_and_preserves_pruning() {
        for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
            let mut r = typed_fixture(format, LIB, TYPED_SOURCE);
            let target = element(&mut r, OWNED);
            for (name, expected) in [
                ("structural::owned", true),
                ("structural::another", true),
                ("inherited::owned", true),
                ("narrowed::owned", false),
                ("structural::plain", false),
            ] {
                let source = element(&mut r, name);
                assert_eq!(
                    r.conforms_with_implied(source, target),
                    expected,
                    "{format:?}: {name}"
                );
                let edges = direct(&mut r, source);
                let count = r.b.elements.len();
                assert_eq!(direct(&mut r, source), edges);
                assert_eq!(r.b.elements.len(), count);
            }
            let owner = element(&mut r, "narrowed");
            let narrow = element(&mut r, "Narrow");
            assert_eq!(r.feature_type_report(owner).types, Ok(vec![narrow]));
        }
        let source = TYPED_SOURCE.replace(" subsets Objects::objects", "");
        let mut r = typed_fixture(GraphFormat::CanonicalV3, LIB, &source);
        let target = element(&mut r, OWNED);
        let source = element(&mut r, "inherited::owned");
        assert!(r.conforms_with_implied(source, target));
    }

    #[test]
    fn typed_step_planning_excludes_future_typing_and_comparison_dependencies() {
        let mut r = typed_fixture(GraphFormat::CanonicalV3, LIB, TYPED_SOURCE);
        let source = element(&mut r, "structural::owned");
        r.implied_relationships(source);
        let direct = element(&mut r, "structural").0;
        let inherited = element(&mut r, "inherited").0;
        let seed = element(&mut r, "seed").0;
        let narrowed = element(&mut r, "narrowed").0;
        let narrow = element(&mut r, "Narrow").0;
        let query = |r: &mut ResolvedModel, changing: HashSet<usize>, steps: &mut usize| {
            super::super::feature_type_projection::structural_owners_for_planning(
                &mut r.b,
                &[direct, inherited, narrowed],
                Arc::new(changing),
                steps,
            )
        };
        let baseline = query(&mut r, HashSet::new(), &mut 0).unwrap();
        assert_eq!(baseline.get(&direct), Some(&true));
        assert_eq!(baseline.get(&inherited), Some(&true));
        assert_eq!(baseline.get(&narrowed), Some(&false));
        let changed = query(&mut r, HashSet::from([seed]), &mut 0).unwrap();
        assert_eq!(changed.get(&direct), Some(&true));
        assert!(!changed.contains_key(&inherited));
        let changed = query(&mut r, HashSet::from([narrow]), &mut 0).unwrap();
        assert!(!changed.contains_key(&narrowed));
        let mut exhausted = crate::eval::MAX_STEPS;
        assert!(query(&mut r, HashSet::new(), &mut exhausted).is_err());
        assert_eq!(query(&mut r, HashSet::new(), &mut 0).unwrap(), baseline);
    }

    fn typed_checked(r: &mut ResolvedModel, steps: &mut usize) -> Option<TypedRequirements> {
        let changing =
            r.b.elements
                .iter()
                .take(r.b.explicit_len())
                .enumerate()
                .filter_map(|(i, e)| typed_candidate(e).then_some(i))
                .collect();
        let raw = StoredStructure::for_query(&mut r.b, steps)?;
        typed_requirements(&mut r.b, Arc::new(changing), &raw, steps)
    }

    #[test]
    fn typed_step_unknown_predicates_and_invalid_library_roles_do_not_construct() {
        for (source, incomplete) in [
            ("feature owner:Missing {composite step owned;}", true),
            // A finite typing closure through a subsetting cycle can still be
            // complete. Cyclic spelling alone does not imply missing evidence.
            (
                "feature owner subsets peer {composite step owned;} feature peer subsets owner;",
                false,
            ),
        ] {
            let mut r = typed_fixture(GraphFormat::CanonicalV3, LIB, source);
            let child = element(&mut r, "owner::owned");
            let result = typed_checked(&mut r, &mut 0).unwrap();
            assert_eq!(result.incomplete.contains(&child.0), incomplete, "{source}");
            assert!(!result.edges.iter().any(|&(source, _)| source == child.0));
            let target = element(&mut r, OWNED);
            assert!(!r.conforms_with_implied(child, target));
        }
        for change in 0..3 {
            let mut r = typed_fixture(GraphFormat::CanonicalV3, LIB, TYPED_SOURCE);
            let child = element(&mut r, "structural::owned");
            let target = element(&mut r, OWNED);
            let id = r.element_id(target);
            match change {
                0 => r.b.elements[target.0].ty = "Feature",
                1 => {
                    r.b.lib_qnames =
                        r.b.lib_qnames
                            .iter()
                            .filter(|(key, _)| *key != id)
                            .cloned()
                            .collect::<Vec<_>>()
                            .into()
                }
                2 => {
                    let other = element(&mut r, "inherited::owned");
                    r.b.lib_qnames.push((
                        r.element_id(other),
                        OWNED.split("::").map(str::to_owned).collect(),
                    ));
                }
                _ => unreachable!(),
            }
            let result = typed_checked(&mut r, &mut 0).unwrap();
            assert!(result.incomplete.contains(&child.0), "change {change}");
            assert!(!result.edges.iter().any(|&(source, _)| source == child.0));
            assert!(!direct(&mut r, child).contains(&id));
        }
    }

    #[test]
    fn typed_step_predicates_recheck_current_membership_flags_types_and_budget() {
        for change in 0..5 {
            let mut r = typed_fixture(GraphFormat::CanonicalV3, LIB, TYPED_SOURCE);
            let child = element(&mut r, "structural::owned");
            let owner = element(&mut r, "structural");
            assert!(
                typed_checked(&mut r, &mut 0)
                    .unwrap()
                    .edges
                    .iter()
                    .any(|&(s, _)| s == child.0)
            );
            let membership = r.b.elements[child.0].owning_relationship.unwrap();
            match change {
                0 => r.b.elements[child.0].owning_relationship = None,
                1 => r.b.set(
                    membership,
                    "ownedMemberFeature",
                    json!({"@id":r.element_id(owner)}),
                ),
                2 => r.b.set(child.0, "isComposite", json!(false)),
                3 => r.b.set(child.0, "isComposite", json!("invalid")),
                4 => {
                    let narrow = element(&mut r, "Narrow");
                    let typing = r.b.elements[owner.0]
                        .owned_relationships
                        .iter()
                        .copied()
                        .find(|&i| r.b.elements[i].ty == "FeatureTyping")
                        .unwrap();
                    r.b.set(typing, "type", json!({"@id":r.element_id(narrow)}));
                }
                _ => unreachable!(),
            }
            assert!(
                typed_checked(&mut r, &mut 0)
                    .is_none_or(|p| !p.edges.iter().any(|&(s, _)| s == child.0)),
                "change {change}"
            );
        }
        let mut r = typed_fixture(GraphFormat::CanonicalV3, LIB, TYPED_SOURCE);
        let child = element(&mut r, "structural::owned");
        let count = r.b.elements.len();
        let mut exhausted = crate::eval::MAX_STEPS;
        assert!(typed_checked(&mut r, &mut exhausted).is_none());
        assert_eq!(r.b.elements.len(), count);
        assert!(
            typed_checked(&mut r, &mut 0)
                .unwrap()
                .edges
                .iter()
                .any(|&(s, _)| s == child.0)
        );
        assert_eq!(r.b.elements.len(), count);
    }

    #[test]
    fn typed_step_construction_survives_prepared_library_replay() {
        use crate::prepared::PreparedLibrary;
        let mut library = Model::with_graph_format(GraphFormat::CanonicalV3);
        library.add_library_source("step-roles.kerml", LIB);
        let prepared = library.prepare_library().unwrap();
        let decoded =
            Arc::new(PreparedLibrary::from_bytes(&prepared.to_bytes(84).unwrap(), 84).unwrap());
        let mut expected = None;
        for library in [prepared, decoded] {
            let mut m = Model::with_graph_format(GraphFormat::CanonicalV3);
            library.install(&mut m).unwrap();
            m.add_source("typed.kerml", TYPED_SOURCE);
            let mut r = ResolvedModel::build(&m);
            let source = element(&mut r, "inherited::owned");
            let target = element(&mut r, OWNED);
            assert!(r.conforms_with_implied(source, target));
            let edges = direct(&mut r, source);
            if let Some(previous) = &expected {
                assert_eq!(&edges, previous);
            } else {
                expected = Some(edges);
            }
        }
    }
    fn element(r: &mut ResolvedModel, name: &str) -> ElementRef {
        r.resolve_qualified(name).unwrap()
    }
    fn direct(r: &mut ResolvedModel, source: ElementRef) -> Vec<uuid::Uuid> {
        r.implied_relationships(source)
            .into_iter()
            .filter_map(|rel| {
                r.b.elements[rel.0]
                    .props
                    .get("subsettedFeature")
                    .and_then(|v| v.as_reference())
            })
            .collect()
    }
    fn requirements(
        r: &mut ResolvedModel,
        source: ElementRef,
        steps: &mut usize,
    ) -> Option<Vec<&'static str>> {
        let raw = StoredStructure::for_query(&mut r.b, steps)?;
        metaclass_requirements(
            &mut r.b,
            source.0,
            &raw,
            &mut TypeRelations::default(),
            steps,
        )
    }

    #[test]
    fn contextual_step_metaclasses_construct_required_paths_and_minimize_self_edges() {
        for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
            let mut r = fixture(format);
            for (name, roles) in [
                ("B::enclosed", vec![ENCLOSED]),
                ("B::owned", vec![ENCLOSED, SUBPERFORMANCES]),
                ("outer::enclosed", vec![ENCLOSED]),
                ("outer::owned", vec![ENCLOSED, SUBPERFORMANCES]),
                ("S::plain", vec![]),
                ("S::owned", vec![OWNED]),
                ("F::enclosed", vec![ENCLOSED]),
                ("P::enclosed", vec![ENCLOSED]),
                ("unowned", vec![]),
            ] {
                let source = element(&mut r, name);
                assert_eq!(
                    requirements(&mut r, source, &mut 0),
                    Some(roles.clone()),
                    "{format:?}: {name}"
                );
                let generic = element(&mut r, "Performances::performances");
                assert!(r.conforms_with_implied(source, generic));
                for role in ROLES {
                    let target = element(&mut r, role);
                    assert_eq!(
                        r.conforms_with_implied(source, target),
                        roles.contains(&role),
                        "{format:?}: {name}: {role}"
                    );
                }
                assert!(r.explicit_specializations(source).is_empty());
                let first = direct(&mut r, source);
                let count = r.b.elements.len();
                assert_eq!(direct(&mut r, source), first);
                assert_eq!(r.b.elements.len(), count);
            }
            for role in ROLES {
                let source = element(&mut r, role);
                let id = r.element_id(source);
                assert!(!direct(&mut r, source).contains(&id), "self edge at {role}");
            }
        }
    }

    #[test]
    fn contextual_step_positive_requirements_recheck_ownership_flags_and_budget() {
        for change in 0..5 {
            let mut r = fixture(GraphFormat::CanonicalV3);
            let source = element(&mut r, "B::owned");
            assert_eq!(
                requirements(&mut r, source, &mut 0),
                Some(vec![ENCLOSED, SUBPERFORMANCES])
            );
            let member = r.b.elements[source.0].owning_relationship.unwrap();
            let owner = element(&mut r, "B");
            match change {
                0 => r.b.elements[source.0].owning_relationship = None,
                1 => r.b.set(
                    member,
                    "ownedMemberFeature",
                    json!({"@id": r.element_id(owner)}),
                ),
                2 => {
                    r.b.set(member, "owningType", json!({"@id": r.element_id(source)}))
                }
                3 => r.b.set(member, "target", json!([])),
                4 => {
                    let rows = r.b.elements[owner.0]
                        .owned_relationships
                        .iter()
                        .copied()
                        .filter(|&rel| rel != member)
                        .collect::<Vec<_>>();
                    r.b.elements[owner.0].owned_relationships = rows.into();
                }
                _ => unreachable!(),
            }
            assert!(
                requirements(&mut r, source, &mut 0).is_none_or(|roles| roles.is_empty()),
                "change {change}"
            );
        }
        let mut r = fixture(GraphFormat::CanonicalV3);
        let source = element(&mut r, "B::owned");
        let mut exhausted = crate::eval::MAX_STEPS;
        assert!(requirements(&mut r, source, &mut exhausted).is_none());
        assert_eq!(
            requirements(&mut r, source, &mut 0),
            Some(vec![ENCLOSED, SUBPERFORMANCES])
        );
        r.b.set(source.0, "isComposite", json!("invalid"));
        assert_eq!(requirements(&mut r, source, &mut 0), Some(vec![ENCLOSED]));
        r.b.set(source.0, "isComposite", json!(false));
        assert_eq!(requirements(&mut r, source, &mut 0), Some(vec![ENCLOSED]));
        let owner = element(&mut r, "B");
        r.b.elements[owner.0].ty = "Structure";
        r.b.set(source.0, "isComposite", json!(true));
        assert_eq!(requirements(&mut r, source, &mut 0), Some(vec![OWNED]));
    }

    #[test]
    fn contextual_step_roles_require_loaded_canonical_step_identities() {
        for (source_name, role) in [
            ("B::enclosed", ENCLOSED),
            ("B::owned", SUBPERFORMANCES),
            ("S::owned", OWNED),
        ] {
            for change in 0..3 {
                let mut r = fixture(GraphFormat::CanonicalV3);
                let source = element(&mut r, source_name);
                let target = element(&mut r, role);
                let id = r.element_id(target);
                match change {
                    0 => r.b.elements[target.0].ty = "Feature",
                    1 => {
                        r.b.lib_qnames =
                            r.b.lib_qnames
                                .iter()
                                .filter(|(key, _)| *key != id)
                                .cloned()
                                .collect::<Vec<_>>()
                                .into();
                    }
                    2 => {
                        let user = element(&mut r, "unowned");
                        r.b.lib_qnames.push((
                            r.element_id(user),
                            role.split("::").map(str::to_owned).collect(),
                        ));
                    }
                    _ => unreachable!(),
                }
                assert!(
                    !direct(&mut r, source).contains(&id),
                    "{role}: change {change}"
                );
                let generic = element(&mut r, "Performances::performances");
                assert!(r.conforms_with_implied(source, generic));
            }
        }
    }

    fn nested_reference_fixture(library: &str) -> (ResolvedModel, ElementRef, ElementRef) {
        let mut m = Model::with_graph_format(GraphFormat::CanonicalV3);
        for (name, text) in [
            ("step-roles.kerml", library),
            (
                "triggers.kerml",
                "standard library package Triggers {function TriggerWhen specializes Performances::Evaluation {in condition;in receiver;return result;}}",
            ),
        ] {
            let result = m.add_library_source(name, text);
            assert!(
                result.diagnostics.is_empty(),
                "{name}: {:?}",
                result.diagnostics
            );
        }
        for (name, text) in [
            (
                "value.kerml",
                "classifier A {feature valueMember;} feature n:A;",
            ),
            ("trigger.sysml", "action a accept when n;"),
        ] {
            let result = m.add_source(name, text);
            assert!(
                result.diagnostics.is_empty(),
                "{name}: {:?}",
                result.diagnostics
            );
        }
        let mut r = ResolvedModel::build(&m);
        let trigger = r
            .user_elements()
            .find(|&e| r.element_type(e) == "TriggerInvocationExpression")
            .unwrap();
        r.implied_relationships(trigger);
        let argument = r
            .invocation_binding_report(trigger)
            .arguments
            .unwrap()
            .bindings[0]
            .value;
        let crate::json::Derived::Value(referent) = r.derived(argument, "referent") else {
            panic!("missing reference")
        };
        let source = referent.element().unwrap();
        assert_eq!(r.element_type(source), "FeatureReferenceExpression");
        let result = ElementRef(
            r.b.semantic_ownership
                .as_ref()
                .unwrap()
                .result(source.0)
                .unwrap()
                .feature,
        );
        (r, source, result)
    }

    #[test]
    fn nested_reference_result_requires_the_constructed_enclosure_path() {
        use crate::json::TypeInputIssue;
        let (mut r, source, result) = nested_reference_fixture(LIB);
        let member = element(&mut r, "A::valueMember");
        let enclosed = element(&mut r, ENCLOSED);
        assert!(r.conforms_with_implied(source, enclosed));
        assert_eq!(
            reference_requirements(&mut r.b, source.0, &mut TypeRelations::default(), &mut 0),
            Some(vec![ENCLOSED])
        );
        let count = r.b.elements.len();
        assert_eq!(
            r.type_feature_report(result).projections.unwrap().features,
            [member]
        );
        assert_eq!(
            r.type_feature_report_with_budget(result, crate::eval::MAX_STEPS)
                .projections,
            Err(TypeInputIssue::WorkLimit)
        );
        assert_eq!(
            r.type_feature_report(result).projections.unwrap().features,
            [member]
        );
        assert_eq!(r.b.elements.len(), count);
        // An otherwise complete canonical result does not stand in for the
        // required contextual library path on its source expression.
        let missing = LIB
            .replace(
                "step enclosedPerformances:Performance subsets performances;",
                "step other:Performance subsets performances;",
            )
            .replace("subsets enclosedPerformances", "subsets other");
        let (mut r, _, result) = nested_reference_fixture(&missing);
        assert_eq!(
            r.type_feature_report(result).projections,
            Err(TypeInputIssue::IncompleteProvider)
        );
    }

    #[test]
    fn nested_reference_context_rejects_positional_and_mutated_ownership() {
        for change in 0..7 {
            let (mut r, source, result) = nested_reference_fixture(LIB);
            assert!(r.type_feature_report(result).projections.is_ok());
            let membership = r.b.elements[source.0].owning_relationship.unwrap();
            let owner = TypeRelations::default()
                .owning_type(&mut r.b, source.0, &mut 0)
                .unwrap()
                .unwrap();
            match change {
                0 => r.b.set(source.0, "direction", json!("in")),
                1 => r.b.elements[membership].ty = "ResultExpressionMembership",
                2 => r.b.elements[membership].ty = "ParameterMembership",
                3 => r.b.set(source.0, "isComposite", json!(true)),
                4 => r.b.set(source.0, "isComposite", json!("invalid")),
                5 => r.b.set(
                    membership,
                    "ownedMemberFeature",
                    json!({"@id":r.b.elements[owner].id}),
                ),
                6 => r.b.elements[source.0].owning_relationship = None,
                _ => unreachable!(),
            }
            assert!(
                reference_requirements(&mut r.b, source.0, &mut TypeRelations::default(), &mut 0)
                    .is_none(),
                "change {change}"
            );
            assert!(
                r.type_feature_report(result).projections.is_err(),
                "change {change}"
            );
        }
    }

    #[test]
    fn contextual_step_construction_survives_prepared_library_replay() {
        use crate::prepared::PreparedLibrary;
        use std::sync::Arc;
        let mut library = Model::with_graph_format(GraphFormat::CanonicalV3);
        library.add_library_source("step-roles.kerml", LIB);
        let prepared = library.prepare_library().unwrap();
        let decoded =
            Arc::new(PreparedLibrary::from_bytes(&prepared.to_bytes(83).unwrap(), 83).unwrap());
        let mut expected = None;
        for library in [prepared, decoded] {
            let mut m = Model::with_graph_format(GraphFormat::CanonicalV3);
            library.install(&mut m).unwrap();
            m.add_source("steps.kerml", SOURCE);
            let mut r = ResolvedModel::build(&m);
            let source = element(&mut r, "B::owned");
            let target = element(&mut r, SUBPERFORMANCES);
            assert!(r.conforms_with_implied(source, target));
            let edges = direct(&mut r, source);
            if let Some(previous) = &expected {
                assert_eq!(&edges, previous);
            } else {
                expected = Some(edges);
            }
        }
    }
}

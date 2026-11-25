//! Trigger argument predicates over the same checked model used by library readers.
use super::{
    ElementRef, InvocationBindingIssue, ResolvedModel, TypeInputIssue, membership_evidence,
    semantic::certified_types::Stamp,
    semantic_ownership,
    structural_index::StoredStructure,
    type_relations::{RelationFact, TypeRelations},
};
use crate::metaclass::conforms;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TriggerArgumentIssue {
    InvalidElement,
    UnsupportedConfiguration,
    IncompleteArguments(InvocationBindingIssue),
    IncompleteResult(TypeInputIssue),
    IncompleteProvider,
    StaleEvidence,
    WorkLimit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct TriggerArgumentReport {
    pub rule: Option<&'static str>,
    /// The conditional argument rule only, not validity of the whole invocation.
    /// Missing evidence is an error, never a successful or failed predicate.
    pub valid: Result<bool, TriggerArgumentIssue>,
    pub steps: usize,
}
use TriggerArgumentIssue::*;
fn charge(steps: &mut usize, n: usize) -> Result<(), TriggerArgumentIssue> {
    *steps = steps.saturating_add(n);
    if *steps > crate::eval::MAX_STEPS {
        Err(WorkLimit)
    } else {
        Ok(())
    }
}
fn truth(fact: RelationFact) -> Result<bool, TriggerArgumentIssue> {
    match fact {
        RelationFact::Yes => Ok(true),
        RelationFact::No => Ok(false),
        RelationFact::Unknown => Err(IncompleteProvider),
    }
}
impl ResolvedModel {
    /// Check the active after/at/when argument rule using ordered invocation
    /// arguments, checked expression results and current library identities.
    /// Canonical invocation construction must already have been prepared.
    /// This query does not publish graph rows or use explicit-only check::Facts.
    pub fn trigger_argument_report(&mut self, receiver: ElementRef) -> TriggerArgumentReport {
        self.trigger_argument_report_with_budget(receiver, 0)
    }
    pub(in crate::json) fn trigger_argument_report_with_budget(
        &mut self,
        receiver: ElementRef,
        initial: usize,
    ) -> TriggerArgumentReport {
        let mut steps = initial;
        let kind = self
            .b
            .elements
            .get(receiver.0)
            .filter(|e| e.ty == "TriggerInvocationExpression")
            .and_then(|e| e.props.get("kind"))
            .and_then(|v| v.as_str());
        let rule = match kind {
            Some("after") => Some("validateTriggerInvocationExpressionAfterArgument"),
            Some("at") => Some("validateTriggerInvocationExpressionAtArgument"),
            Some("when") => Some("validateTriggerInvocationExpressionWhenArgument"),
            _ => None,
        };
        let valid = if self.b.elements.get(receiver.0).is_none() {
            Err(InvalidElement)
        } else if let Some(rule) = rule {
            self.checked_trigger_argument(receiver, rule, &mut steps)
        } else {
            Err(UnsupportedConfiguration)
        };
        TriggerArgumentReport {
            rule,
            valid: if steps > crate::eval::MAX_STEPS {
                Err(WorkLimit)
            } else {
                valid
            },
            steps,
        }
    }
    fn trigger_library_target(
        &mut self,
        name: &str,
        owned: bool,
        steps: &mut usize,
    ) -> Result<usize, TriggerArgumentIssue> {
        let membership =
            super::operations::namespaces::resolution::checked_global(self, name, steps)
                .map_err(|_| IncompleteProvider)?
                .ok_or(IncompleteProvider)?;
        if membership.member >= self.b.lib_boundary
            || !conforms(self.b.elements[membership.member].ty, "Type")
            || (owned
                && !conforms(
                    self.b.elements[membership.relationship].ty,
                    "OwningMembership",
                ))
        {
            return Err(IncompleteProvider);
        }
        Ok(membership.member)
    }
    fn trigger_expression_result(
        &mut self,
        expression: usize,
        steps: &mut usize,
    ) -> Result<Option<usize>, TriggerArgumentIssue> {
        let report = self.expression_result_report_with_budget(ElementRef(expression), *steps);
        *steps = report.steps;
        report
            .result
            .map(|v| v.map(|e| e.0))
            .map_err(IncompleteResult)
    }
    fn checked_trigger_argument(
        &mut self,
        receiver: ElementRef,
        rule: &str,
        steps: &mut usize,
    ) -> Result<bool, TriggerArgumentIssue> {
        charge(steps, 1)?;
        let arguments = self
            .b
            .checked_invocation_bindings(receiver.0, steps, None)
            .map_err(IncompleteArguments)?;
        let stamp = Stamp::capture(self);
        let result = (|| {
            let Some(argument) = arguments.bindings.first() else {
                return Ok(false);
            };
            let mut expression = argument.value.0;
            if rule == "validateTriggerInvocationExpressionWhenArgument" {
                if !conforms(self.b.elements[expression].ty, "FeatureReferenceExpression") {
                    return Ok(false);
                }
                let Some(referent) = self.checked_trigger_referent(expression, steps)? else {
                    return Ok(false);
                };
                if !conforms(self.b.elements[referent].ty, "Expression") {
                    return Ok(false);
                }
                expression = referent;
            }
            let Some(result) = self.trigger_expression_result(expression, steps)? else {
                return Ok(false);
            };
            let required = match rule {
                "validateTriggerInvocationExpressionAfterArgument" => {
                    "Quantities::ScalarQuantityValue"
                }
                "validateTriggerInvocationExpressionAtArgument" => "Time::TimeInstantValue",
                _ => "ScalarValues::Boolean",
            };
            let target = self.trigger_library_target(required, false, steps)?;
            let mut relations = TypeRelations::default();
            if !truth(relations.specializes(&mut self.b, result, target, steps))? {
                return Ok(false);
            }
            if rule != "validateTriggerInvocationExpressionAfterArgument" {
                return Ok(true);
            }
            let reference =
                self.trigger_library_target("Quantities::TensorQuantityValue::mRef", true, steps)?;
            let unit = self.trigger_library_target("ISQBase::DurationUnit", false, steps)?;
            let sequence = self
                .b
                .checked_type_memberships_with_relations(result, steps, &mut relations)
                .map_err(IncompleteResult)?;
            let features = self
                .b
                .project_type_features(&sequence, steps)
                .map_err(IncompleteResult)?;
            let mut unknown = false;
            for feature in features.features {
                if !sequence
                    .transitively_redefines(feature.0, reference, steps)
                    .map_err(IncompleteResult)?
                {
                    continue;
                }
                match relations.specializes(&mut self.b, feature.0, unit, steps) {
                    RelationFact::Yes => return Ok(true),
                    RelationFact::No => {}
                    RelationFact::Unknown => unknown = true,
                }
            }
            if unknown {
                Err(IncompleteProvider)
            } else {
                Ok(false)
            }
        })();
        if !stamp.current(self) {
            return Err(StaleEvidence);
        }
        result
    }
    // Unlike additive reference-result construction, the when wrapper owns its
    // referent expression. Both owning and nonowning memberships use the common
    // reciprocal endpoint reader, in exact source order.
    fn checked_trigger_referent(
        &mut self,
        expression: usize,
        steps: &mut usize,
    ) -> Result<Option<usize>, TriggerArgumentIssue> {
        let raw = StoredStructure::for_query(&mut self.b, steps).ok_or(IncompleteProvider)?;
        if !raw.ids_unique
            || raw.annotations_incomplete
            || self.b.metadata_associations_incomplete
            || !raw
                .membership_domains(&self.b, steps)
                .ok_or(IncompleteProvider)?
                .owner_complete(expression)
            || raw.metadata_annotation_targets.contains(&expression)
            || self
                .b
                .metadata_of
                .get(&expression)
                .is_some_and(|v| !v.is_empty())
        {
            return Err(IncompleteProvider);
        }
        let rows = semantic_ownership::owned_relationships(&self.b, expression)
            .ok_or(IncompleteProvider)?;
        charge(steps, rows.len())?;
        let rows: Vec<_> = rows.iter().collect();
        let mut seen = HashSet::new();
        let mut first = None;
        for relationship in rows {
            if !seen.insert(relationship)
                || semantic_ownership::checked_relationship_carrier(
                    &self.b,
                    &raw,
                    relationship,
                    steps,
                ) != Some(Some(expression))
            {
                return Err(IncompleteProvider);
            }
            let kind = self.b.elements[relationship].ty;
            if !conforms(kind, "Membership") {
                continue;
            }
            let member =
                membership_evidence::member(&self.b, &raw, expression, relationship, steps)
                    .ok_or(IncompleteProvider)?;
            if first.is_none() && !conforms(kind, "ParameterMembership") {
                first = Some(member);
            }
        }
        Ok(first.filter(|&e| conforms(self.b.elements[e].ty, "Feature")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{GraphFormat, Model},
        prepared::PreparedLibrary,
    };
    use serde_json::json;
    use std::sync::{Arc, OnceLock};

    fn library() -> Arc<PreparedLibrary> {
        static LIBRARY: OnceLock<Arc<PreparedLibrary>> = OnceLock::new();
        Arc::clone(LIBRARY.get_or_init(|| {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../spec-refs/SysML-v2-Release/sysml.library");
            let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
            assert_eq!(model.load_library_dir(&root).unwrap(), 94);
            model.prepare_library().unwrap()
        }))
    }
    const SOURCE: &str =
        "package P {attribute delay:ISQ::DurationValue; action a accept after delay;}";
    fn fixture(source: &str) -> (ResolvedModel, ElementRef) {
        let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
        library().install(&mut model).unwrap();
        assert!(
            model
                .add_source("trigger.sysml", source)
                .diagnostics
                .is_empty()
        );
        let mut r = ResolvedModel::build(&model);
        let trigger = r
            .user_elements()
            .find(|&e| r.element_type(e) == "TriggerInvocationExpression")
            .unwrap();
        r.ensure_implied();
        (r, trigger)
    }

    // Construct abstract inputs before any implied suffix is published. These
    // are deliberately not text-syntax or whole-model validity assertions.
    fn abstract_trigger(kind: &str) -> (ResolvedModel, ElementRef) {
        let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
        library().install(&mut model).unwrap();
        let parsed = model.add_source(
            "abstract-trigger.kerml",
            "package P {
                expr condition {return result:ScalarValues::Boolean;}
                expr number {return result:ScalarValues::Integer;}
                feature ready:ScalarValues::Boolean;
                package NotFeature;
            }",
        );
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let mut r = ResolvedModel::build(&model);
        let owner = r.resolve_qualified("P").unwrap().0;
        let membership = r.b.new_relationship("FeatureMembership", owner, "trigger");
        let trigger =
            r.b.new_owned_element("TriggerInvocationExpression", membership, "trigger");
        r.b.set(trigger, "kind", json!(kind));
        (r, ElementRef(trigger))
    }

    fn abstract_parameter(r: &mut ResolvedModel, trigger: ElementRef) -> usize {
        let membership =
            r.b.new_relationship("ParameterMembership", trigger.0, "argument");
        let parameter = r.b.new_owned_element("Feature", membership, "argument");
        r.b.set(parameter, "direction", json!("in"));
        parameter
    }

    fn abstract_when_wrapper(r: &mut ResolvedModel, trigger: ElementRef) -> usize {
        let parameter = abstract_parameter(r, trigger);
        let callee = r.resolve_qualified("Triggers::TriggerWhen").unwrap();
        let input = r.type_input_report(callee).inputs.unwrap()[0];
        let redefinition = r.b.new_relationship("Redefinition", parameter, "mapping");
        r.b.set(
            redefinition,
            "redefiningFeature",
            json!({"@id":r.b.elements[parameter].id}),
        );
        r.b.set(
            redefinition,
            "redefinedFeature",
            json!({"@id":r.element_id(input)}),
        );
        let value = r.b.new_relationship("FeatureValue", parameter, "value");
        r.b.new_owned_element("FeatureReferenceExpression", value, "wrapper")
    }

    fn abstract_reference(
        r: &mut ResolvedModel,
        wrapper: usize,
        target: usize,
        name: &str,
    ) -> usize {
        let membership = r.b.new_relationship("Membership", wrapper, name);
        r.b.set(
            membership,
            "memberElement",
            json!({"@id":r.b.elements[target].id}),
        );
        membership
    }

    #[test]
    fn abstract_empty_trigger_arguments_are_false_without_inventing_defaults() {
        for kind in ["after", "at", "when"] {
            for unvalued_slot in [false, true] {
                let (mut r, trigger) = abstract_trigger(kind);
                if unvalued_slot {
                    abstract_parameter(&mut r, trigger);
                }
                r.ensure_implied();
                let before = r.b.elements.len();
                let report = r.invocation_binding_report(trigger).arguments.unwrap();
                assert!(!report.inputs.is_empty());
                assert!(report.bindings.is_empty());
                let report = r.trigger_argument_report(trigger);
                assert_eq!(
                    report.rule,
                    Some(match kind {
                        "after" => "validateTriggerInvocationExpressionAfterArgument",
                        "at" => "validateTriggerInvocationExpressionAtArgument",
                        _ => "validateTriggerInvocationExpressionWhenArgument",
                    })
                );
                assert_eq!(report.valid, Ok(false), "{kind}, slot={unvalued_slot}");
                assert_eq!(r.b.elements.len(), before);
            }
            let (mut r, trigger) = abstract_trigger(kind);
            let parameter = abstract_parameter(&mut r, trigger);
            // A present FeatureValue without its owned Expression is damaged
            // evidence, not a proof that the parameter has no valuation.
            r.b.new_relationship("FeatureValue", parameter, "broken-value");
            r.ensure_implied();
            assert!(r.invocation_binding_report(trigger).arguments.is_err());
            assert!(matches!(
                r.trigger_argument_report(trigger).valid,
                Err(IncompleteArguments(_))
            ));
        }
    }

    #[test]
    fn abstract_when_referent_selection_preserves_first_membership_and_parameter_exclusion() {
        for (targets, expected, valid) in [
            (vec![], None, false),
            (vec!["condition", "number"], Some("condition"), true),
            (vec!["number", "condition"], Some("number"), false),
            (vec!["ready", "condition"], Some("ready"), false),
            (vec!["NotFeature", "condition"], None, false),
        ] {
            let (mut r, trigger) = abstract_trigger("when");
            let wrapper = abstract_when_wrapper(&mut r, trigger);
            // Both ordinary input and return memberships are excluded from
            // referent selection even when they precede the target membership.
            for (kind, direction) in [
                ("ParameterMembership", "in"),
                ("ReturnParameterMembership", "out"),
            ] {
                let membership = r.b.new_relationship(kind, wrapper, kind);
                let feature = r.b.new_owned_element("Feature", membership, "parameter");
                r.b.set(feature, "direction", json!(direction));
            }
            for (index, name) in targets.iter().enumerate() {
                let target = r.resolve_qualified(&format!("P::{name}")).unwrap().0;
                abstract_reference(&mut r, wrapper, target, &format!("member-{index}"));
            }
            let expected =
                expected.map(|name| r.resolve_qualified(&format!("P::{name}")).unwrap().0);
            r.ensure_implied();
            assert_eq!(
                r.checked_trigger_referent(wrapper, &mut 0),
                Ok(expected),
                "{targets:?}"
            );
            let bindings = r.invocation_binding_report(trigger).arguments.unwrap();
            assert_eq!(bindings.bindings.len(), 1);
            assert_eq!(bindings.bindings[0].value, ElementRef(wrapper));
            let before = r.b.elements.len();
            assert_eq!(
                r.trigger_argument_report(trigger).valid,
                Ok(valid),
                "{targets:?}"
            );
            assert_eq!(r.b.elements.len(), before);
        }
    }

    #[test]
    fn abstract_when_requires_a_reference_wrapper_and_complete_member_evidence() {
        let (mut r, trigger) = abstract_trigger("when");
        let argument = abstract_when_wrapper(&mut r, trigger);
        r.b.elements[argument].ty = "LiteralBoolean";
        r.b.set(argument, "value", json!(true));
        r.ensure_implied();
        assert_eq!(
            r.invocation_binding_report(trigger)
                .arguments
                .unwrap()
                .bindings[0]
                .value,
            ElementRef(argument)
        );
        // A Boolean result alone cannot replace the required reference wrapper.
        assert_eq!(r.trigger_argument_report(trigger).valid, Ok(false));

        let (mut r, trigger) = abstract_trigger("when");
        let wrapper = abstract_when_wrapper(&mut r, trigger);
        let first = r.resolve_qualified("P::condition").unwrap().0;
        let later = r.resolve_qualified("P::number").unwrap().0;
        abstract_reference(&mut r, wrapper, first, "first");
        let later_membership = abstract_reference(&mut r, wrapper, later, "later");
        r.ensure_implied();
        assert_eq!(r.trigger_argument_report(trigger).valid, Ok(true));
        // A successful first member does not authorize ignoring contradictory
        // endpoints in another member of the claimed complete collection.
        r.b.set(
            later_membership,
            "target",
            json!([{"@id":r.b.elements[first].id}]),
        );
        assert_eq!(
            r.checked_trigger_referent(wrapper, &mut 0),
            Err(IncompleteProvider)
        );
        assert!(r.trigger_argument_report(trigger).valid.is_err());
    }

    #[test]
    fn expression_owned_return_domains_require_complete_current_owners() {
        for change in 0..6 {
            let (mut r, trigger) = abstract_trigger("when");
            let wrapper = abstract_when_wrapper(&mut r, trigger);
            let expression = r.resolve_qualified("P::number").unwrap();
            let result = r.resolve_qualified("P::number::result").unwrap();
            abstract_reference(&mut r, wrapper, expression.0, "number");
            r.ensure_implied();
            assert_eq!(
                r.expression_result_report(expression).result,
                Ok(Some(result))
            );
            assert!(r.type_feature_report(result).projections.is_ok());
            assert_eq!(r.trigger_argument_report(trigger).valid, Ok(false));
            let membership = r.b.elements[result.0].owning_relationship.unwrap();
            match change {
                0 => r.b.set(result.0, "direction", json!("in")),
                1 => r.b.elements[result.0].owning_relationship = None,
                2 => r.b.set(
                    membership,
                    "ownedMemberParameter",
                    json!({"@id":r.element_id(expression)}),
                ),
                3 => r.b.set(expression.0, "isComposite", json!(true)),
                4 => r.b.metadata_associations_incomplete = true,
                5 => {
                    let missing = r.resolve_qualified("Performances::evaluations").unwrap();
                    r.b.set(missing.0, "declaredName", json!("UnavailableEvaluations"));
                }
                _ => unreachable!(),
            }
            assert!(
                r.type_feature_report(result).projections.is_err(),
                "change {change}"
            );
            let report = r.trigger_argument_report(trigger);
            let selected = r.expression_result_report(expression);
            if change == 0 {
                // The return's direction constraint is independent of result
                // selection and this conditional Boolean type predicate.
                // Its Integer result still proves the trigger rule false.
                assert_eq!(selected.result, Ok(Some(result)));
                assert_eq!(report.valid, Ok(false));
            } else {
                assert!(
                    report.valid.is_err(),
                    "change {change}: {report:?}, {selected:?}"
                );
            }
        }
    }

    #[test]
    fn authored_step_and_expression_parameters_share_signature_evidence() {
        let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
        library().install(&mut model).unwrap();
        let parsed = model.add_source(
            "parameters.kerml",
            "
            behavior B {in input:ScalarValues::Integer;out output:ScalarValues::Integer;}
            step s:B {in input:ScalarValues::Integer;out output:ScalarValues::Integer;}
            function F {in input:ScalarValues::Integer;return result:ScalarValues::Integer;}
            expr e:F {in input:ScalarValues::Integer;return result:ScalarValues::Integer;}",
        );
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let mut r = ResolvedModel::build(&model);
        let boolean = r.resolve_qualified("ScalarValues::Boolean").unwrap();
        let integer = r.resolve_qualified("ScalarValues::Integer").unwrap();
        let parameters = ["s::input", "s::output", "e::input", "e::result"]
            .map(|name| r.resolve_qualified(name).unwrap());
        r.ensure_implied();
        for parameter in parameters {
            assert!(
                r.type_feature_report(parameter).projections.is_ok(),
                "{parameter:?}"
            );
            let mut relations = TypeRelations::default();
            assert_eq!(
                relations.specializes(&mut r.b, parameter.0, integer.0, &mut 0),
                RelationFact::Yes
            );
            assert_eq!(
                relations.specializes(&mut r.b, parameter.0, boolean.0, &mut 0),
                RelationFact::No
            );
        }
        let parameter = parameters[3];
        let expression = r.resolve_qualified("e").unwrap();
        assert_eq!(
            r.expression_result_report(expression).result,
            Ok(Some(parameter))
        );
        assert_eq!(
            r.type_input_report_with_budget(parameter, crate::eval::MAX_STEPS)
                .inputs,
            Err(TypeInputIssue::WorkLimit)
        );
        assert!(r.type_feature_report(parameter).projections.is_ok());
    }

    #[test]
    fn trigger_argument_budget_is_cumulative_retryable_and_read_only() {
        let (mut r, trigger) = fixture(SOURCE);
        let before = r.b.elements.len();
        let report = r.trigger_argument_report(trigger);
        assert_eq!(report.valid, Ok(true), "{} steps", report.steps);
        assert!(report.steps > 0 && report.steps < crate::eval::MAX_STEPS);
        // The first read also initializes shared structural indexes. Measure
        // the warm proof before testing a partially consumed query allowance.
        let warm = r.trigger_argument_report(trigger);
        assert_eq!(warm.valid, Ok(true));
        // Hash-map traversal can change the precise work count. A large seed
        // must survive every nested query, without requiring deterministic gas.
        let seed = crate::eval::MAX_STEPS / 2;
        let seeded = r.trigger_argument_report_with_budget(trigger, seed);
        assert_eq!(seeded.valid, Ok(true));
        assert!(seeded.steps > seed && seeded.steps <= crate::eval::MAX_STEPS);
        for initial in [crate::eval::MAX_STEPS, crate::eval::MAX_STEPS - 1] {
            assert_eq!(
                r.trigger_argument_report_with_budget(trigger, initial)
                    .valid,
                Err(WorkLimit)
            );
        }
        assert_eq!(r.trigger_argument_report(trigger).valid, Ok(true));
        assert_eq!(r.b.elements.len(), before);
    }

    #[test]
    fn trigger_arguments_refuse_stale_results_memberships_and_library_evidence() {
        for change in 0..10 {
            let (mut r, trigger) = fixture(SOURCE);
            assert_eq!(r.trigger_argument_report(trigger).valid, Ok(true));
            let argument = r
                .invocation_binding_report(trigger)
                .arguments
                .unwrap()
                .bindings[0]
                .clone();
            let result = r
                .expression_result_report(argument.value)
                .result
                .unwrap()
                .unwrap();
            let unit = r.resolve_qualified("ISQBase::DurationUnit").unwrap();
            let reference = r.resolve_qualified("ISQ::DurationValue::mRef").unwrap();
            match change {
                0 => r.b.set(trigger.0, "kind", json!("unknown")),
                1 => r.b.set(trigger.0, "kind", json!("when")),
                2 => r.b.elements[result.0].owning_relationship = None,
                3 => r.b.set(argument.parameter.0, "direction", json!("out")),
                4 => r.b.metadata_associations_incomplete = true,
                5 => r.b.elements[unit.0].id = r.b.elements[result.0].id,
                6 => r.b.elements[reference.0].owning_relationship = None,
                7 => r.b.elements[argument.value.0].owning_relationship = None,
                8 => r.b.set(argument.value.0, "isComposite", json!(true)),
                9 => {
                    r.b.set(unit.0, "declaredName", json!("RenamedDurationUnit"))
                }
                _ => unreachable!(),
            }
            let before = r.b.elements.len();
            assert!(
                r.trigger_argument_report(trigger).valid.is_err(),
                "mutation {change}"
            );
            assert_eq!(r.b.elements.len(), before);
        }
    }

    #[test]
    fn trigger_library_resolution_preserves_global_ambiguity_and_missing_library_uncertainty() {
        let (mut r, trigger) = fixture(&format!(
            "{SOURCE} package Quantities {{attribute def ScalarQuantityValue;}}"
        ));
        assert!(r.trigger_argument_report(trigger).valid.is_err());
        let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
        model.add_source("missing.sysml", "action a accept after 1;");
        let mut r = ResolvedModel::build(&model);
        let trigger = r
            .user_elements()
            .find(|&e| r.element_type(e) == "TriggerInvocationExpression")
            .unwrap();
        r.ensure_implied();
        assert!(r.trigger_argument_report(trigger).valid.is_err());
        let (mut r, trigger) = fixture("action a accept when true;");
        assert_eq!(r.trigger_argument_report(trigger).valid, Ok(true));
        let boolean = r.resolve_qualified("ScalarValues::Boolean").unwrap();
        r.b.set(boolean.0, "declaredName", json!("RenamedBoolean"));
        assert!(r.trigger_argument_report(trigger).valid.is_err());
    }

    #[test]
    fn when_wrapper_uses_the_first_nonparameter_membership_and_reciprocal_ownership() {
        for change in 0..3 {
            let (mut r, trigger) = fixture("action a accept when true;");
            assert_eq!(r.trigger_argument_report(trigger).valid, Ok(true));
            let wrapper = r
                .invocation_binding_report(trigger)
                .arguments
                .unwrap()
                .bindings[0]
                .value;
            let referent = r
                .checked_trigger_referent(wrapper.0, &mut 0)
                .unwrap()
                .unwrap();
            let relationship = r.b.elements[referent].owning_relationship.unwrap();
            match change {
                0 => r.b.elements[referent].owning_relationship = None,
                1 => r.b.set(
                    relationship,
                    "ownedMemberElement",
                    json!({"@id":r.element_id(trigger)}),
                ),
                2 => r.b.elements[wrapper.0].owned_relationships = Vec::new().into(),
                _ => unreachable!(),
            }
            assert!(
                r.checked_trigger_referent(wrapper.0, &mut 0).is_err(),
                "mutation {change}"
            );
            assert!(r.trigger_argument_report(trigger).valid.is_err());
        }
    }

    #[test]
    fn trigger_predicates_replay_prepared_and_serialized_libraries() {
        let prepared = library();
        let decoded =
            Arc::new(PreparedLibrary::from_bytes(&prepared.to_bytes(87).unwrap(), 87).unwrap());
        let mut previous = None;
        for prepared in [prepared, decoded] {
            let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
            prepared.install(&mut model).unwrap();
            model.add_source("trigger.sysml", SOURCE);
            let mut r = ResolvedModel::build(&model);
            let trigger = r
                .user_elements()
                .find(|&e| r.element_type(e) == "TriggerInvocationExpression")
                .unwrap();
            r.ensure_implied();
            assert_eq!(r.trigger_argument_report(trigger).valid, Ok(true));
            let id = r.element_id(trigger);
            if let Some(previous) = previous {
                assert_eq!(id, previous);
            }
            previous = Some(id);
        }
    }
}

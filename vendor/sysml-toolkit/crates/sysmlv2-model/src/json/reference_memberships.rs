//! Construction evidence for the complete admitted reference-expression source.
//! Selection still belongs to the common ordered membership reducer.
use super::{Builder, membership_evidence, semantic_ownership, structural_index::StoredStructure};
use crate::metaclass::conforms;

#[cfg(test)]
mod foreign_members_tests;

pub(super) struct ReferenceMembers {
    pub(super) binding: usize,
}

impl Builder {
    pub(super) fn reference_membership_domain(
        &mut self,
        source: usize,
        steps: &mut usize,
    ) -> Option<ReferenceMembers> {
        *steps = steps.saturating_add(1);
        if *steps > crate::eval::MAX_STEPS
            || self.elements.get(source)?.ty != "FeatureReferenceExpression"
        {
            return None;
        }
        let view = self.semantic_ownership.as_ref()?;
        if !view.matches_suffix(self) {
            return None;
        }
        let result = view.result(source)?;
        if self.complete_generated_reference_result(result.feature, steps)? != source {
            return None;
        }
        let raw = StoredStructure::for_query(self, steps)?;
        let referent = membership_evidence::first_unowned_member(
            self,
            &raw,
            source,
            "ParameterMembership",
            "Feature",
            None,
            steps,
        )?;
        let rows = semantic_ownership::owned_relationships(self, source)?;
        *steps = steps.saturating_add(rows.len());
        if *steps > crate::eval::MAX_STEPS {
            return None;
        }
        let rows: Vec<_> = rows.iter().collect();
        let mut binding = None;
        for relationship in rows {
            let kind = self.elements.get(relationship)?.ty;
            // This activates a distinct result-expression binding obligation.
            if conforms(kind, "ResultExpressionMembership") {
                return None;
            }
            if !conforms(kind, "Membership") {
                continue;
            }
            let member = membership_evidence::member(self, &raw, source, relationship, steps)?;
            if self.elements[member].ty != "BindingConnector" {
                continue;
            }
            let view = self.semantic_ownership.as_ref()?;
            if !view.contains(member) || view.source_anchor(member) != source {
                continue;
            }
            if kind != "OwningMembership"
                || view.generated_relationship_owner(relationship) != Some(source)
                || binding.replace(member).is_some()
                || self.elements[member]
                    .props
                    .get("direction")
                    .is_some_and(|v| !v.is_null())
                || ["isEnd", "isComposite", "isVariable", "isPortion"]
                    .iter()
                    .any(|key| {
                        self.elements[member]
                            .props
                            .get(key)
                            .is_some_and(|v| v.as_bool() != Some(false))
                    })
                || super::owned_results::binary_binding_targets(self, &raw, member, steps)?
                    != [referent, result.feature]
            {
                return None;
            }
        }
        // Generated binding ancestry/featuring is a distinct obligation. Its
        // OwningMembership gives it no owningType and therefore no outer end,
        // parameter or result redefinition role. The Type provider still reads
        // every current owned/inverse redefinition edge before reducing members.
        Some(ReferenceMembers { binding: binding? })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        json::{ClosurePolicy, ElementRef, PropertyError, ResolvedModel, TypeInputIssue},
        model::{GraphFormat, Model},
    };
    use serde_json::json;

    const LIBRARY: &str = "standard library package Base {classifier Anything; feature things:Anything;}
standard library package Occurrences {class Occurrence specializes Base::Anything; feature occurrences:Occurrence subsets Base::things;}
standard library package Performances {
 behavior Performance specializes Occurrences::Occurrence;
 step performances:Performance subsets Occurrences::occurrences;
 function Evaluation specializes Performance {return result;}
 expr evaluations:Evaluation subsets performances;
}";

    fn resolved(model: &Model) -> (ResolvedModel, ElementRef, ElementRef) {
        let mut r = ResolvedModel::build(model);
        let source = r
            .user_elements()
            .find(|&e| r.element_type(e) == "FeatureReferenceExpression")
            .unwrap();
        r.ensure_implied();
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

    fn fixture(format: GraphFormat, source: &str) -> (ResolvedModel, ElementRef, ElementRef) {
        let mut model = Model::with_graph_format(format);
        assert!(
            model
                .add_library_source("reference-library.kerml", LIBRARY)
                .diagnostics
                .is_empty()
        );
        assert!(
            model
                .add_source("reference.kerml", source)
                .diagnostics
                .is_empty()
        );
        resolved(&model)
    }

    #[test]
    fn checked_reference_result_uses_semantic_memberships_without_publishing_rows() {
        for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
            for source in [
                "feature value; feature selected=value;",
                "classifier C {feature value;feature unrelated;} feature selected=C::value;",
            ] {
                let (mut r, expression, result) = fixture(format, source);
                let before = r.b.elements.len();
                assert_eq!(
                    r.expression_result_report(expression).result,
                    Ok(Some(result)),
                    "{format:?}: {source}"
                );
                let features = r
                    .type_feature_report(expression)
                    .projections
                    .unwrap()
                    .features;
                assert_eq!(features, [result]);
                r.set_closure_policy(ClosurePolicy::Passthrough);
                assert_eq!(
                    r.property(expression, "result"),
                    Err(PropertyError::Approximate)
                );
                r.set_closure_policy(ClosurePolicy::Closure {
                    include_implied: true,
                });
                assert_eq!(
                    r.property(expression, "result"),
                    Ok(json!({"@id":r.element_id(result)}))
                );
                assert_eq!(r.b.elements.len(), before);
            }
        }
    }

    #[test]
    fn foreign_return_dependencies_are_scheduled_without_becoming_inherited_features() {
        let source = "function Parent {in input;return output;}
            function Child specializes Parent {in next;return answer;}
            feature selected=Child::answer;";
        for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
            let (mut r, expression, result) = fixture(format, source);
            let foreign = r.resolve_qualified("Child::answer").unwrap();
            let before = r.b.elements.len();
            assert_eq!(
                r.expression_result_report(expression).result,
                Ok(Some(result)),
                "{format:?}"
            );
            let features = r
                .type_feature_report(expression)
                .projections
                .unwrap()
                .features;
            assert_eq!(features, [result], "{format:?}");
            assert!(!features.contains(&foreign));
            assert_eq!(
                r.expression_result_report_with_budget(expression, crate::eval::MAX_STEPS)
                    .result,
                Err(TypeInputIssue::WorkLimit)
            );
            assert_eq!(
                r.expression_result_report(expression).result,
                Ok(Some(result))
            );
            assert_eq!(r.b.elements.len(), before);

            // The foreign owner is a prerequisite in both formats. A warm
            // successful query must not survive damage to that ownership.
            r.b.elements[foreign.0].owning_relationship.take().unwrap();
            assert!(
                r.expression_result_report(expression).result.is_err(),
                "{format:?}"
            );
            assert_eq!(r.b.elements.len(), before);
            // Model edits currently rebuild the resolved snapshot. Exercise that
            // supported lifecycle rather than silently repairing private rows.
            let (mut rebuilt, expression, rebuilt_result) = fixture(format, source);
            assert_eq!(
                rebuilt.expression_result_report(expression).result,
                Ok(Some(rebuilt_result))
            );
            assert_eq!(rebuilt.element_id(rebuilt_result), r.element_id(result));
        }
    }

    #[test]
    fn borrowed_members_suppress_redefined_inputs_through_foreign_positional_closure() {
        let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
        model.add_library_source("reference-library.kerml", LIBRARY);
        model.add_source(
            "aliases.kerml",
            "function Signature {in a;in b;}
            function Foreign specializes Signature {in x;}
            function Aliases {alias borrowed for Foreign::x;}
            function R specializes Signature, Aliases;",
        );
        assert!(!model.has_errors());
        let mut r = ResolvedModel::build(&model);
        let receiver = r.resolve_qualified("R").unwrap();
        let b = r.resolve_qualified("Signature::b").unwrap();
        let foreign = r.resolve_qualified("Foreign::x").unwrap();
        let before = r.b.elements.len();
        // The nonowning alias is not itself an input. Its referent nevertheless
        // redefines Signature::a positionally and suppresses that inherited member.
        assert_eq!(r.type_input_report(receiver).inputs, Ok(vec![b]));
        let features = r.type_feature_report(receiver).projections.unwrap();
        assert!(!features.features.contains(&foreign));
        assert_eq!(r.b.elements.len(), before);
    }

    #[test]
    fn borrowed_members_require_complete_foreign_ownership_and_ancestry() {
        for declaration in [
            "function Foreign specializes Missing {in x;}",
            "function Foreign specializes Cycle {in x;} function Cycle specializes Foreign;",
        ] {
            let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
            model.add_library_source("reference-library.kerml", LIBRARY);
            model.add_source(
                "aliases.kerml",
                &format!(
                    "{declaration}
                function R {{alias borrowed for Foreign::x;}}"
                ),
            );
            let mut r = ResolvedModel::build(&model);
            let receiver = r.resolve_qualified("R").unwrap();
            assert!(
                r.type_input_report(receiver).inputs.is_err(),
                "{declaration}"
            );
        }
        let (mut r, expression, result) = fixture(
            GraphFormat::CanonicalV3,
            "classifier C {feature value;} feature selected=C::value;",
        );
        assert_eq!(
            r.expression_result_report(expression).result,
            Ok(Some(result))
        );
        let value = r.resolve_qualified("C::value").unwrap();
        r.b.elements[value.0].owning_relationship = None;
        assert!(r.expression_result_report(expression).result.is_err());
    }

    #[test]
    fn reference_memberships_recheck_binding_endpoints_ownership_and_source_evidence() {
        for change in 0..12 {
            let (mut r, source, result) = fixture(
                GraphFormat::CanonicalV3,
                "feature value;feature other;feature selected=value;",
            );
            assert_eq!(r.expression_result_report(source).result, Ok(Some(result)));
            let binding =
                r.b.reference_membership_domain(source.0, &mut 0)
                    .unwrap()
                    .binding;
            let membership = r.b.elements[binding].owning_relationship.unwrap();
            let end_membership = r.b.elements[binding].owned_relationships[0];
            let end = r.b.elements[end_membership].children[0];
            let reference = r.b.elements[end].owned_relationships[0];
            let other = r.resolve_qualified("other").unwrap();
            match change {
                0 => r.b.set(
                    reference,
                    "referencedFeature",
                    json!({"@id":r.element_id(other)}),
                ),
                1 => {
                    r.b.set(reference, "general", json!({"@id":r.element_id(other)}))
                }
                2 => r.b.elements[end].owning_relationship = None,
                3 => r.b.elements[membership].ty = "FeatureMembership",
                4 => r.b.elements[binding].owned_relationships = vec![end_membership].into(),
                5 => r.b.set(binding, "direction", json!("in")),
                6 => r.b.set(source.0, "isComposite", json!(true)),
                7 => r.b.metadata_associations_incomplete = true,
                8 => r.b.elements[result.0].owning_relationship = None,
                9 => r.b.elements[end_membership].children = vec![other.0].into(),
                10 => r.b.elements[reference].ty = "Subsetting",
                11 => r.b.set(binding, "isEnd", json!(true)),
                _ => unreachable!(),
            }
            let before = r.b.elements.len();
            assert!(
                r.expression_result_report(source).result.is_err(),
                "mutation {change}"
            );
            r.set_closure_policy(ClosurePolicy::Closure {
                include_implied: true,
            });
            assert!(
                r.property(source, "result").is_err(),
                "property mutation {change}"
            );
            assert_eq!(r.b.elements.len(), before);
        }
    }

    #[test]
    fn reference_result_budget_refusal_is_retryable_and_missing_evidence_is_not_absence() {
        let (mut r, source, result) = fixture(
            GraphFormat::CanonicalV3,
            "feature value;feature selected=value;",
        );
        assert_eq!(
            r.expression_result_report_with_budget(source, crate::eval::MAX_STEPS)
                .result,
            Err(TypeInputIssue::WorkLimit)
        );
        assert_eq!(r.expression_result_report(source).result, Ok(Some(result)));
        let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
        model.add_source("missing.kerml", "feature value;feature selected=value;");
        let (mut r, source, _) = resolved(&model);
        assert!(r.expression_result_report(source).result.is_err());
    }

    #[test]
    fn checked_reference_results_survive_prepared_library_and_disk_cache_replay() {
        use crate::prepared::PreparedLibrary;
        use std::sync::Arc;
        let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
        model.add_library_source("reference-library.kerml", LIBRARY);
        let prepared = model.prepare_library().unwrap();
        let decoded =
            Arc::new(PreparedLibrary::from_bytes(&prepared.to_bytes(86).unwrap(), 86).unwrap());
        let mut previous = None;
        for prepared in [prepared, decoded] {
            let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
            prepared.install(&mut model).unwrap();
            model.add_source("reference.kerml", "feature value;feature selected=value;");
            let (mut r, source, result) = resolved(&model);
            assert_eq!(r.expression_result_report(source).result, Ok(Some(result)));
            let identities = (r.element_id(source), r.element_id(result));
            if let Some(previous) = previous {
                assert_eq!(identities, previous);
            }
            previous = Some(identities);
        }
    }
}

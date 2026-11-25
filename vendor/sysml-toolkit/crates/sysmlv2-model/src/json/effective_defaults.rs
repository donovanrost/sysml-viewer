//! Effective FeatureValue selection shared by constructor and invocation
//! consumers. The caller supplies a complete current Type.feature projection
//! and exact supplied-feature identities. Selection never constructs bindings.
use super::{
    Builder, ElementRef, membership_evidence, semantic::certified_types::Stamp, semantic_ownership,
    structural_index::StoredStructure, type_relations,
};
use crate::metaclass::conforms;
use std::collections::{HashMap, HashSet};

/// An effective omitted default, selected by checked redefinition identity.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct EffectiveDefaultBinding {
    pub feature: ElementRef,
    pub feature_with_value: ElementRef,
    pub valuation: ElementRef,
    pub value: ElementRef,
}
/// An incomplete default dependency is not complete absence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EffectiveDefaultIssue {
    InvalidRelationship,
    IncompleteProvider,
    UnsupportedConfiguration,
    StaleEvidence,
    WorkLimit,
}
use EffectiveDefaultIssue::*;
/// Completed per-feature selections shared only within one caller's batch.
/// Partial/cyclic results are never cached; source or publication changes drop
/// all retained identities before reuse, including complete absence.
#[derive(Default)]
pub(super) struct EffectiveDefaultBatch {
    stamp: Option<Stamp>,
    memo: HashMap<usize, Option<(usize, usize, usize)>>,
}
fn charge(steps: &mut usize, n: usize) -> Result<(), EffectiveDefaultIssue> {
    *steps = steps.saturating_add(n);
    if *steps > crate::eval::MAX_STEPS {
        Err(WorkLimit)
    } else {
        Ok(())
    }
}

impl Builder {
    pub(super) fn checked_effective_defaults(
        &mut self,
        features: &[ElementRef],
        supplied: &HashSet<usize>,
        steps: &mut usize,
        batch: &mut EffectiveDefaultBatch,
    ) -> Result<Vec<EffectiveDefaultBinding>, EffectiveDefaultIssue> {
        let retained = self
            .retained_typing_edges(steps)
            .ok_or(IncompleteProvider)?;
        let raw = StoredStructure::for_query(self, steps).ok_or(IncompleteProvider)?;
        let inverse = raw.typing(self, steps).ok_or(IncompleteProvider)?;
        if !raw.ids_unique
            || raw.annotations_incomplete
            || inverse.sources_incomplete
            || self.metadata_associations_incomplete
        {
            return Err(IncompleteProvider);
        }
        let stamp = Stamp::capture_builder(self);
        if !batch
            .stamp
            .as_ref()
            .is_some_and(|stamp| stamp.current_builder(self))
        {
            charge(steps, batch.memo.capacity())?;
            batch.memo.clear();
            batch.stamp = Some(Stamp::capture_builder(self));
        }
        let mut defaults = Vec::new();
        for &feature in features {
            charge(steps, 1)?;
            if supplied.contains(&feature.0) {
                continue;
            }
            if let Some((owner, valuation, value)) = self.effective_default(
                feature.0,
                &raw,
                &retained,
                &mut batch.memo,
                &mut HashSet::new(),
                steps,
            )? {
                defaults.push(EffectiveDefaultBinding {
                    feature,
                    feature_with_value: ElementRef(owner),
                    valuation: ElementRef(valuation),
                    value: ElementRef(value),
                });
            }
        }
        if !stamp.current_builder(self) {
            return Err(StaleEvidence);
        }
        Ok(defaults)
    }
    #[allow(clippy::too_many_arguments)]
    fn effective_default(
        &mut self,
        feature: usize,
        raw: &StoredStructure,
        retained: &super::implied::RetainedTypings,
        memo: &mut HashMap<usize, Option<(usize, usize, usize)>>,
        active: &mut HashSet<usize>,
        steps: &mut usize,
    ) -> Result<Option<(usize, usize, usize)>, EffectiveDefaultIssue> {
        charge(steps, 1)?;
        if let Some(value) = memo.get(&feature) {
            return Ok(*value);
        }
        if active.len() >= super::MAX_RESOLUTION_DEPTH || !active.insert(feature) {
            return Err(IncompleteProvider);
        }
        if raw.metadata_annotation_targets.contains(&feature)
            || self
                .metadata_of
                .get(&feature)
                .is_some_and(|m| !m.is_empty())
            || !raw
                .membership_domains(self, steps)
                .ok_or(IncompleteProvider)?
                .owner_complete(feature)
        {
            return Err(IncompleteProvider);
        }
        let valuation =
            membership_evidence::valuation(self, raw, feature, steps).ok_or(InvalidRelationship)?;
        let inverse = raw.typing(self, steps).ok_or(IncompleteProvider)?;
        let owned =
            semantic_ownership::owned_relationships(self, feature).ok_or(IncompleteProvider)?;
        charge(steps, owned.len())?;
        let owned: Vec<_> = owned.iter().collect();
        let mut targets = Vec::new();
        let mut seen = HashSet::new();
        for relationship in owned.iter().copied().chain(
            inverse
                .relationships
                .get(&feature)
                .into_iter()
                .flatten()
                .copied(),
        ) {
            charge(steps, 1)?;
            if !seen.insert(relationship) {
                continue;
            }
            let kind = self.elements[relationship].ty;
            if conforms(kind, "Conjugation") {
                return Err(UnsupportedConfiguration);
            }
            if !conforms(kind, "Redefinition") {
                continue;
            }
            if !owned.contains(&relationship)
                || semantic_ownership::checked_relationship_carrier(self, raw, relationship, steps)
                    != Some(Some(feature))
                || !self.static_chain_row_current(relationship)
                || !self.result_redefinition_row_current(relationship)
            {
                return Err(InvalidRelationship);
            }
            let target = type_relations::endpoint_with_carrier(
                self,
                feature,
                Some(feature),
                relationship,
                &["specific", "subsettingFeature", "redefiningFeature"],
                &["general", "subsettedFeature", "redefinedFeature"],
                "Feature",
                steps,
            )
            .ok_or(InvalidRelationship)?;
            targets.push(target);
        }
        if let Some(implied) = retained.get(&feature) {
            charge(steps, implied.len())?;
            for &(kind, id) in implied {
                if conforms(kind, "Redefinition") {
                    targets.push(raw.element_for_uuid(self, id).ok_or(IncompleteProvider)?);
                }
            }
        }
        // A complete shared positional plan can imply redefinitions before
        // physical materialization. These identities belong to the same
        // provider used by Type.feature, never an independent lexical search.
        if let Some(plan) = self.effective_dynamic_plan() {
            if plan.positional.incomplete.contains(&feature) {
                return Err(IncompleteProvider);
            }
            if let Some(implied) = plan.positional.targets.get(&feature) {
                charge(steps, implied.len())?;
                targets.extend(implied);
            }
        } else if let Some(plan) = &self.positional_redefinitions {
            if plan.incomplete.contains(&feature) {
                return Err(IncompleteProvider);
            }
            if let Some(implied) = plan.targets.get(&feature) {
                charge(steps, implied.len())?;
                targets.extend(implied);
            }
        } else {
            return Err(IncompleteProvider);
        }
        let mut inherited = HashSet::new();
        for target in targets {
            if let Some(candidate) =
                self.effective_default(target, raw, retained, memo, active, steps)?
            {
                inherited.insert(candidate);
            }
        }
        let selected = if let Some((membership, value)) = valuation {
            self.elements[membership]
                .props
                .get("isDefault")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                .then_some((feature, membership, value))
        } else if inherited.len() == 1 {
            inherited.into_iter().next()
        } else {
            // KerML 8.4.4.11: inherited defaults apply only when their set
            // contains one FeatureValue identity. Distinct defaults establish
            // absence, even when their value expressions happen to be equal.
            // An incomplete path has already returned an error above.
            None
        };
        active.remove(&feature);
        memo.insert(feature, selected);
        Ok(selected)
    }
}

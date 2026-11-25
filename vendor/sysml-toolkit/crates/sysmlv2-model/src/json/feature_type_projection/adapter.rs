//! Repository typing through checked family applicability and current semantic
//! membership/positional evidence. Unsupported domains remain explicit.
use super::{Comparison, Evidence, Incomplete, Inputs, Projection};
use crate::json::{
    Builder, semantic_ownership,
    structural_index::{StoredStructure, StoredTyping},
    type_relations::{
        CompositionOwner, FeatureRequirements, RelationFact, TypeRelations, endpoint_with_carrier,
    },
};
use crate::metaclass::conforms;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub(super) struct RepositoryTyping<'a> {
    changing_specializations: Option<Arc<HashSet<usize>>>,
    builder: &'a mut Builder,
    context: super::ProjectionContext,
    contextual_owners: HashMap<usize, CompositionOwner>,
    structure: Arc<StoredStructure>,
    typing: Arc<StoredTyping>,
    ownership: Option<Arc<semantic_ownership::SemanticOwnership>>,
    metadata_generation: Option<Arc<()>>,
    publication: crate::json::publication::Revision,
    relations: TypeRelations,
    retained: Arc<crate::json::implied::RetainedTypings>,
    retained_plan: Option<Arc<crate::json::implied::SupportedImpliedSpecializations>>,
}
fn charge(steps: &mut usize, amount: usize) -> Result<(), Incomplete> {
    *steps = steps.saturating_add(amount);
    if *steps > crate::eval::MAX_STEPS {
        Err(Incomplete::Budget)
    } else {
        Ok(())
    }
}
impl<'a> RepositoryTyping<'a> {
    pub(super) fn new(builder: &'a mut Builder, steps: &mut usize) -> Result<Self, Incomplete> {
        if builder.positional_planning {
            return Err(Incomplete::UnsupportedProjection);
        }
        let retained = builder.retained_typing_edges(steps).ok_or({
            if *steps > crate::eval::MAX_STEPS {
                Incomplete::Budget
            } else {
                Incomplete::UnsupportedProjection
            }
        })?;
        let structure = StoredStructure::for_query(builder, steps).ok_or({
            if *steps > crate::eval::MAX_STEPS {
                Incomplete::Budget
            } else {
                Incomplete::InvalidRelationship
            }
        })?;
        if !structure.ids_unique {
            return Err(Incomplete::InvalidElement);
        }
        let typing = structure.typing(builder, steps).ok_or({
            if *steps > crate::eval::MAX_STEPS {
                Incomplete::Budget
            } else {
                Incomplete::InvalidRelationship
            }
        })?;
        let retained_plan = builder.supported_implied.clone();
        let ownership = builder.semantic_ownership.clone();
        let metadata_generation = builder.metadata_association_generation.clone();
        let publication = builder.publication.revision();
        let context = super::ProjectionContext::default();
        let relations = TypeRelations::with_structure(Arc::clone(&structure))
            .with_projection_context(context.clone());
        Ok(Self {
            changing_specializations: None,
            context: context.clone(),
            contextual_owners: HashMap::new(),
            publication,
            builder,
            ownership,
            metadata_generation,
            structure,
            typing,
            retained,
            retained_plan,
            relations,
        })
    }
    /// Complete projections independent of every prospective specialization
    /// writer. The publisher must audit its final writes against this same set.
    pub(super) fn for_planning(
        builder: &'a mut Builder,
        changing: Arc<HashSet<usize>>,
        steps: &mut usize,
    ) -> Result<Self, Incomplete> {
        Self::with_context(
            builder,
            Some(changing),
            super::ProjectionContext::default(),
            steps,
        )
    }
    pub(super) fn with_context(
        builder: &'a mut Builder,
        changing: Option<Arc<HashSet<usize>>>,
        context: super::ProjectionContext,
        steps: &mut usize,
    ) -> Result<Self, Incomplete> {
        let mut reader = Self::new(builder, steps)?;
        reader.relations = match &changing {
            Some(changing) => TypeRelations::with_stable_specializations(
                Arc::clone(&reader.structure),
                Arc::clone(changing),
            ),
            None => TypeRelations::with_structure(Arc::clone(&reader.structure)),
        }
        .with_projection_context(context.clone());
        reader.changing_specializations = changing;
        reader.context = context;
        Ok(reader)
    }
    pub(super) fn has_kind(&self, element: usize, kind: &str) -> bool {
        self.builder
            .elements
            .get(element)
            .is_some_and(|e| conforms(e.ty, kind))
    }
    pub(super) fn project(&mut self, feature: usize, steps: &mut usize) -> Projection {
        let entry = charge(steps, 1).and_then(|()| self.context.enter(feature));
        let _entry = match entry {
            Ok(entry) => entry,
            Err(issue) => {
                return Projection {
                    candidates: Vec::new(),
                    graph_issue: Some(issue),
                    required_issue: None,
                    steps: *steps,
                };
            }
        };
        super::project(
            self,
            feature,
            steps,
            crate::eval::MAX_STEPS,
            crate::json::MAX_RESOLUTION_DEPTH,
        )
    }
    pub(super) fn owner_types(
        &mut self,
        owner: usize,
        steps: &mut usize,
    ) -> Result<CompositionOwner, Incomplete> {
        charge(steps, 1)?;
        if !self.current() {
            return Err(Incomplete::UnsupportedProjection);
        }
        if let Some(certificate) = self.contextual_owners.get(&owner) {
            return Ok(*certificate);
        }
        let projection = self.project(owner, steps);
        if let Some(issue) = projection.graph_issue.or(projection.required_issue) {
            return Err(issue);
        }
        charge(steps, projection.candidates.len())?;
        let certificate = CompositionOwner {
            object: projection
                .candidates
                .iter()
                .any(|&ty| self.has_kind(ty, "Structure")),
            occurrence: projection
                .candidates
                .iter()
                .any(|&ty| self.has_kind(ty, "Class")),
        };
        self.contextual_owners.insert(owner, certificate);
        Ok(certificate)
    }
    fn current(&self) -> bool {
        let ownership_current = match (&self.ownership, &self.builder.semantic_ownership) {
            (None, None) => true,
            (Some(stored), Some(current)) => Arc::ptr_eq(stored, current),
            _ => false,
        };
        let metadata_current = match (
            &self.metadata_generation,
            &self.builder.metadata_association_generation,
        ) {
            (None, None) => true,
            (Some(stored), Some(current)) => Arc::ptr_eq(stored, current),
            _ => false,
        };
        let retained_current = match (&self.retained_plan, &self.builder.supported_implied) {
            (None, None) => true,
            (Some(old), Some(current)) => Arc::ptr_eq(old, current),
            _ => false,
        };
        self.structure.is_current(self.builder)
            && ownership_current
            && metadata_current
            && retained_current
            && self
                .publication
                .same_as(&self.builder.publication.revision())
    }
    fn carrier(&self, relationship: usize, steps: &mut usize) -> Result<Option<usize>, Incomplete> {
        let result = semantic_ownership::checked_relationship_carrier(
            self.builder,
            &self.structure,
            relationship,
            steps,
        );
        result.ok_or(Incomplete::InvalidRelationship)
    }
    fn endpoint(
        &mut self,
        source: usize,
        relation: usize,
        kind: &str,
        steps: &mut usize,
    ) -> Result<usize, Incomplete> {
        if !self.builder.static_chain_row_current(relation) {
            return Err(Incomplete::UnsupportedProjection);
        }
        let carrier = self.carrier(relation, steps)?;
        endpoint_with_carrier(
            self.builder,
            source,
            carrier,
            relation,
            &[
                "specific",
                "typedFeature",
                "subsettingFeature",
                "redefiningFeature",
                "referencingFeature",
                "crossingFeature",
            ],
            &[
                "general",
                "type",
                "subsettedFeature",
                "redefinedFeature",
                "referencedFeature",
                "crossedFeature",
            ],
            kind,
            steps,
        )
        .ok_or({
            if *steps > crate::eval::MAX_STEPS {
                Incomplete::Budget
            } else {
                Incomplete::InvalidRelationship
            }
        })
    }
    fn owned_relationships(
        &self,
        feature: usize,
        steps: &mut usize,
    ) -> Result<Vec<usize>, Incomplete> {
        let rows = semantic_ownership::owned_relationships(self.builder, feature)
            .ok_or(Incomplete::InvalidRelationship)?;
        charge(steps, rows.len())?;
        let relationships = rows.iter().collect::<Vec<_>>();
        let mut seen = HashSet::new();
        for &relationship in &relationships {
            if !seen.insert(relationship) || self.carrier(relationship, steps)? != Some(feature) {
                return Err(Incomplete::InvalidRelationship);
            }
        }
        Ok(relationships)
    }
    /// An intentionally narrow but complete applicability audit for obligations
    /// that can change this Feature's typing closure. Every required path is
    /// checked later against the collected graph; this never adds an edge.
    fn requirements(
        &mut self,
        feature: usize,
        owned: &[usize],
        typings: &[usize],
        original: Option<usize>,
        steps: &mut usize,
    ) -> Result<FeatureRequirements, Incomplete> {
        let owner = self
            .relations
            .feature_context_owner(self.builder, feature, &self.structure, steps)
            .ok_or(Incomplete::InvalidRelationship)?;
        let owner_types = owner
            .map(|owner| self.owner_types(owner, steps))
            .transpose()?;
        self.relations
            .feature_requirements(
                self.builder,
                feature,
                owned,
                typings,
                original,
                owner_types,
                steps,
            )
            .map_err(|error| match error {
                super::super::type_relations::FeatureRequirementsFailure::Unsupported => {
                    Incomplete::MissingRequiredFamilies
                }
                super::super::type_relations::FeatureRequirementsFailure::InvalidRelationship => {
                    Incomplete::InvalidRelationship
                }
            })
    }
}
impl Evidence for RepositoryTyping<'_> {
    fn inputs(&mut self, feature: usize, steps: &mut usize) -> Result<Inputs, Incomplete> {
        charge(steps, 1)?;
        if self
            .changing_specializations
            .as_ref()
            .is_some_and(|rows| rows.contains(&feature))
        {
            return Err(Incomplete::UnsupportedProjection);
        }
        if !self.current()
            || (!self.builder.dynamic_evidence_current(feature)
                || !self.builder.result_redefinition_evidence_current(feature))
        {
            return Err(Incomplete::UnsupportedProjection);
        }
        if !conforms(
            self.builder
                .elements
                .get(feature)
                .ok_or(Incomplete::InvalidElement)?
                .ty,
            "Feature",
        ) {
            return Err(Incomplete::InvalidElement);
        }
        let owned = self.owned_relationships(feature, steps)?;
        // Existing checked conjugation helper validates owned cardinality,
        // identity and scalar/generic aliases. No private normalization graph.
        let original = self
            .relations
            .owned_conjugation(self.builder, feature, steps)
            .ok_or(Incomplete::InvalidRelationship)?;
        let inverse = self
            .typing
            .relationships
            .get(&feature)
            .map_or(&[][..], Vec::as_slice);
        charge(steps, inverse.len())?;
        let inverse = inverse.to_vec();
        let mut typings = Vec::new();
        let mut features = Vec::new();
        let mut graph_issue = self
            .typing
            .sources_incomplete
            .then_some(Incomplete::InvalidRelationship);
        for relationship in inverse {
            let kind = self.builder.elements[relationship].ty;
            if conforms(kind, "FeatureTyping") {
                typings.push(self.endpoint(feature, relationship, "Type", steps)?);
            } else if original.is_none()
                && conforms(kind, "Subsetting")
                && !conforms(kind, "CrossSubsetting")
            {
                features.push(self.endpoint(feature, relationship, "Feature", steps)?);
            }
        }
        if let Some(original) = original {
            if conforms(self.builder.elements[original].ty, "Feature") {
                features.push(original);
            }
        }
        let retained = self.retained.get(&feature).map_or(&[][..], Vec::as_slice);
        charge(steps, retained.len())?;
        for &(kind, id) in retained {
            if original.is_some() && !conforms(kind, "FeatureTyping") {
                continue;
            }
            if conforms(kind, "CrossSubsetting") {
                continue;
            }
            let Some(target) = self.builder.element_index_of_uuid(id) else {
                graph_issue.get_or_insert(Incomplete::ExternalEndpoint);
                continue;
            };
            let expected = if conforms(kind, "FeatureTyping") {
                "Type"
            } else {
                "Feature"
            };
            if !conforms(self.builder.elements[target].ty, expected) {
                graph_issue.get_or_insert(Incomplete::InvalidElement);
                continue;
            }
            if expected == "Type" {
                typings.push(target);
            } else {
                features.push(target);
            }
        }
        let requirements = self.requirements(feature, &owned, &typings, original, steps);
        let (required_features, required_typings, required_issue) = match requirements {
            Ok(requirements) => {
                // The same complete membership proof used for applicability
                // supplies semantic positional targets, including generated
                // results without lexical scopes. No second stored-only plan.
                charge(steps, requirements.positional_redefinitions.len())?;
                features.extend(requirements.positional_redefinitions);
                // Reuse the chain certificate required by the same family proof;
                // its common reader validates every link and its ownership.
                if let Some(last) = requirements.chaining_feature {
                    charge(steps, 1)?;
                    features.push(last);
                }
                (requirements.features, requirements.typings, None)
            }
            Err(Incomplete::Budget) => return Err(Incomplete::Budget),
            Err(issue) => (Vec::new(), Vec::new(), Some(issue)),
        };
        charge(steps, 0)?;
        Ok(Inputs {
            typings,
            features,
            graph_issue,
            required_issue,
            required_features,
            required_typings,
        })
    }
    fn specializes(&mut self, specific: usize, general: usize, steps: &mut usize) -> Comparison {
        if !self.current() {
            return Comparison::Unknown;
        }
        match self
            .relations
            .specializes(self.builder, specific, general, steps)
        {
            RelationFact::Yes => Comparison::Yes,
            RelationFact::No => Comparison::No,
            RelationFact::Unknown => Comparison::Unknown,
        }
    }
    fn is_behavior(&self, element: usize) -> bool {
        self.builder
            .elements
            .get(element)
            .is_some_and(|e| conforms(e.ty, "Behavior"))
    }
    fn is_function(&self, element: usize) -> bool {
        self.builder
            .elements
            .get(element)
            .is_some_and(|e| conforms(e.ty, "Function"))
    }
}
#[cfg(test)]
mod readiness_lifecycle_tests {
    use super::*;
    use crate::{json::ResolvedModel, model::Model};
    #[test]
    fn metadata_snapshot_change_refuses_old_proof_and_repair_retries_fresh() {
        let library = "standard library package Base {classifier Anything; feature things:Anything;} standard library package Occurrences {class Occurrence specializes Base::Anything; feature occurrences:Occurrence subsets Base::things;} standard library package Performances {behavior Performance specializes Occurrences::Occurrence; function Evaluation specializes Performance; step performances:Performance subsets Occurrences::occurrences; expr evaluations:Evaluation subsets performances;}";
        let mut model = Model::new();
        assert!(
            model
                .add_library_source("bases.kerml", library)
                .diagnostics
                .is_empty()
        );
        assert!(model.add_source("source.kerml","function F specializes Performances::Evaluation; expr e:F subsets Performances::evaluations;").diagnostics.is_empty());
        let mut r = ResolvedModel::build(&model);
        let feature = r.resolve_qualified("e").unwrap().0;
        let function = r.resolve_qualified("F").unwrap().0;
        let mut proof = RepositoryTyping::new(&mut r.b, &mut 0).unwrap();
        assert_eq!(
            proof.project(feature, &mut 0).function(&proof),
            Ok(Some(function))
        );
        // Reconstructing an equivalent retained table still revokes the old
        // snapshot: unchanged rows/Boolean readiness are not its authority.
        proof.builder.supported_implied = None;
        proof.builder.retained_typing_edges(&mut 0).unwrap();
        assert!(!proof.current());
        assert!(proof.project(feature, &mut 0).function(&proof).is_err());
        drop(proof);
        let mut proof = RepositoryTyping::new(&mut r.b, &mut 0).unwrap();
        assert_eq!(
            proof.project(feature, &mut 0).function(&proof),
            Ok(Some(function))
        );
        proof.builder.metadata_associations_incomplete = true;
        proof.builder.metadata_association_generation = Some(Arc::new(()));
        assert!(!proof.current());
        assert!(proof.project(feature, &mut 0).function(&proof).is_err());
        drop(proof);
        let mut unproved = RepositoryTyping::new(&mut r.b, &mut 0).unwrap();
        let incomplete = unproved.project(feature, &mut 0);
        assert_eq!(
            incomplete.required_issue,
            Some(Incomplete::MissingRequiredFamilies)
        );
        assert!(
            incomplete.function(&unproved).is_err(),
            "metadata also qualifies specialization negatives"
        );
        unproved.builder.metadata_associations_incomplete = false;
        unproved.builder.metadata_association_generation = Some(Arc::new(()));
        drop(unproved);
        let mut repaired = RepositoryTyping::new(&mut r.b, &mut 0).unwrap();
        assert_eq!(
            repaired.project(feature, &mut 0).function(&repaired),
            Ok(Some(function))
        );
    }
}

//! Complete collections for Feature.type, Step.behavior and Expression.function.
//!
//! Candidates are not certified final types when graph/required-family evidence
//! is incomplete. InstantiatedType is deliberately absent from this interface.
mod adapter;
mod report;
pub use report::{FeatureTypeIssue, FeatureTypeReport};
#[cfg(test)]
mod readiness_tests;
#[cfg(test)]
mod repository_tests;
#[cfg(test)]
mod semantic_positional_tests;

use std::collections::{HashMap, HashSet};

/// Query-local recursion boundary shared with negative specialization readers.
/// A nested reader cannot evade it by creating a new type projector. This holds
/// active requests only, never semantic facts, and is independent of model state.
#[derive(Clone, Default)]
pub(super) struct ProjectionContext(std::sync::Arc<std::sync::Mutex<HashSet<usize>>>);
struct ProjectionEntry {
    context: ProjectionContext,
    feature: usize,
}
impl ProjectionContext {
    fn enter(&self, feature: usize) -> Result<ProjectionEntry, Incomplete> {
        let mut active = self
            .0
            .lock()
            .map_err(|_| Incomplete::UnsupportedProjection)?;
        if active.len() >= crate::json::MAX_RESOLUTION_DEPTH {
            return Err(Incomplete::Depth);
        }
        if !active.insert(feature) {
            return Err(Incomplete::UnsupportedProjection);
        }
        Ok(ProjectionEntry {
            context: self.clone(),
            feature,
        })
    }
}
impl Drop for ProjectionEntry {
    fn drop(&mut self) {
        // No lock is held across a projection; unwind and early returns both
        // remove the request so a later independent query can retry.
        self.context
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&self.feature);
    }
}

pub(super) fn contextual_owner_types(
    builder: &mut crate::json::Builder,
    owner: usize,
    changing: Option<std::sync::Arc<HashSet<usize>>>,
    context: ProjectionContext,
    steps: &mut usize,
) -> Result<crate::json::type_relations::CompositionOwner, Incomplete> {
    let mut reader = adapter::RepositoryTyping::with_context(builder, changing, context, steps)?;
    reader.owner_types(owner, steps)
}

/// Evaluate construction predicates with the ordinary complete type reducer.
/// Missing entries are unknown, not false. No candidate model or semantic rows
/// are published; every accepted projection excludes the supplied future writes.
pub(super) fn structural_owners_for_planning(
    builder: &mut crate::json::Builder,
    owners: &[usize],
    changing: std::sync::Arc<HashSet<usize>>,
    steps: &mut usize,
) -> Result<HashMap<usize, bool>, Incomplete> {
    owners_of_kind_for_planning(builder, owners, changing, "Structure", steps)
}

pub(super) fn owners_of_kind_for_planning(
    builder: &mut crate::json::Builder,
    owners: &[usize],
    changing: std::sync::Arc<HashSet<usize>>,
    kind: &str,
    steps: &mut usize,
) -> Result<HashMap<usize, bool>, Incomplete> {
    let mut reader = adapter::RepositoryTyping::for_planning(builder, changing, steps)?;
    let mut complete = HashMap::new();
    for &owner in owners {
        if !charge(steps, 1, crate::eval::MAX_STEPS) {
            return Err(Incomplete::Budget);
        }
        if complete.contains_key(&owner) {
            continue;
        }
        let projection = reader.project(owner, steps);
        if *steps > crate::eval::MAX_STEPS {
            return Err(Incomplete::Budget);
        }
        if !projection.complete() {
            continue;
        }
        if !charge(steps, projection.candidates.len(), crate::eval::MAX_STEPS) {
            return Err(Incomplete::Budget);
        }
        complete.insert(
            owner,
            projection
                .candidates
                .iter()
                .any(|&ty| reader.has_kind(ty, kind)),
        );
    }
    Ok(complete)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Comparison {
    Yes,
    No,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Incomplete {
    Budget,
    Depth,
    InvalidElement,
    InvalidRelationship,
    ExternalEndpoint,
    MissingRequiredFamilies,
    UnsupportedProjection,
    AmbiguousSpecialization,
    InvalidFunctionMultiplicity,
    InvalidBehaviorKind,
}
#[derive(Clone, Debug)]
pub(super) struct Inputs {
    /// Complete direct inverse FeatureTyping targets, in stable identity order.
    pub(super) typings: Vec<usize>,
    /// Complete typingFeatures: non-cross inverse subsettings plus final owned
    /// chain, or only the original Feature when conjugated. Own typings still
    /// count when conjugated; conjugation changes dependencies, not this vector.
    pub(super) features: Vec<usize>,
    pub(super) graph_issue: Option<Incomplete>,
    /// Required implied-family coverage is separate from a complete stored read.
    pub(super) required_issue: Option<Incomplete>,
    /// Audited domain obligations. Each must have a positive witness in the
    /// SAME collected typingFeatures graph, starting from this Feature.
    pub(super) required_features: Vec<usize>,
    pub(super) required_typings: Vec<usize>,
}
pub(super) trait Evidence {
    fn inputs(&mut self, feature: usize, steps: &mut usize) -> Result<Inputs, Incomplete>;
    fn specializes(&mut self, specific: usize, general: usize, steps: &mut usize) -> Comparison;
    fn is_behavior(&self, element: usize) -> bool;
    fn is_function(&self, element: usize) -> bool;
}
#[derive(Clone, Debug)]
pub(super) struct Projection {
    /// Possible surviving types. On incomplete input this is neither a complete
    /// collection nor a proof that each listed candidate actually survives.
    pub(super) candidates: Vec<usize>,
    pub(super) graph_issue: Option<Incomplete>,
    pub(super) required_issue: Option<Incomplete>,
    pub(super) steps: usize,
}
impl Projection {
    pub(super) fn complete(&self) -> bool {
        self.graph_issue.is_none() && self.required_issue.is_none()
    }
    pub(super) fn function<E: Evidence>(&self, evidence: &E) -> Result<Option<usize>, Incomplete> {
        if let Some(issue) = self.graph_issue.or(self.required_issue) {
            return Err(issue);
        }
        let mut function = None;
        for &ty in &self.candidates {
            if !evidence.is_behavior(ty) {
                continue;
            }
            // Expression.function redefines behavior. A Behavior that is not a
            // Function violates the narrowing; filtering it away is unsound.
            if !evidence.is_function(ty) {
                return Err(Incomplete::InvalidBehaviorKind);
            }
            if function.replace(ty).is_some() {
                return Err(Incomplete::InvalidFunctionMultiplicity);
            }
        }
        Ok(function)
    }
    pub(super) fn behaviors<E: Evidence>(&self, evidence: &E) -> Result<Vec<usize>, Incomplete> {
        if let Some(issue) = self.graph_issue.or(self.required_issue) {
            return Err(issue);
        }
        Ok(self
            .candidates
            .iter()
            .copied()
            .filter(|&ty| evidence.is_behavior(ty))
            .collect())
    }
}
fn charge(steps: &mut usize, amount: usize, maximum: usize) -> bool {
    *steps = steps.saturating_add(amount);
    *steps <= maximum
}

fn required_paths(
    graph: &HashMap<usize, Inputs>,
    source: usize,
    input: &Inputs,
    steps: &mut usize,
    maximum: usize,
) -> Result<(), Incomplete> {
    if input.required_features.is_empty() && input.required_typings.is_empty() {
        return Ok(());
    }
    if !charge(
        steps,
        input
            .required_features
            .len()
            .saturating_add(input.required_typings.len()),
        maximum,
    ) {
        return Err(Incomplete::Budget);
    }
    let mut features: HashSet<_> = input.required_features.iter().copied().collect();
    let mut typings: HashSet<_> = input.required_typings.iter().copied().collect();
    let mut visited = HashSet::new();
    let mut todo = vec![source];
    while let Some(feature) = todo.pop() {
        if !charge(steps, 1, maximum) {
            return Err(Incomplete::Budget);
        }
        if !visited.insert(feature) {
            continue;
        }
        features.remove(&feature);
        let row = graph
            .get(&feature)
            .ok_or(Incomplete::MissingRequiredFamilies)?;
        if !charge(
            steps,
            row.features.len().saturating_add(row.typings.len()),
            maximum,
        ) {
            return Err(Incomplete::Budget);
        }
        for target in &row.typings {
            typings.remove(target);
        }
        if features.is_empty() && typings.is_empty() {
            return Ok(());
        }
        todo.extend(row.features.iter().copied());
    }
    Err(Incomplete::MissingRequiredFamilies)
}

/// Ordered closure reads every reachable Feature once. A cycle is finite OCL
/// closure, not an automatic failure. No partial/negative projection is cached.
/// The same caller-owned budget also bounds endpoint and comparison providers.
pub(super) fn project<E: Evidence>(
    evidence: &mut E,
    feature: usize,
    steps: &mut usize,
    maximum: usize,
    depth_limit: usize,
) -> Projection {
    let mut graph_issue = None;
    let mut required_issue = None;
    let mut inputs = HashMap::new();
    let mut input_order = Vec::new();
    let mut seen_features = HashSet::new();
    let mut seen_types = HashSet::new();
    let mut types = Vec::new();
    let mut queue = vec![(feature, 0usize)];
    let mut at = 0;
    while at < queue.len() {
        if !charge(steps, 1, maximum) {
            graph_issue = Some(Incomplete::Budget);
            break;
        }
        let (current, depth) = queue[at];
        at += 1;
        if !seen_features.insert(current) {
            continue;
        }
        if depth > depth_limit {
            graph_issue.get_or_insert(Incomplete::Depth);
            continue;
        }
        let input = match evidence.inputs(current, steps) {
            Ok(input) => input,
            Err(issue) => {
                graph_issue.get_or_insert(issue);
                continue;
            }
        };
        if !charge(
            steps,
            input.typings.len().saturating_add(input.features.len()),
            maximum,
        ) {
            graph_issue = Some(Incomplete::Budget);
            break;
        }
        if let Some(issue) = input.graph_issue {
            graph_issue.get_or_insert(issue);
        }
        if let Some(issue) = input.required_issue {
            required_issue.get_or_insert(issue);
        }
        for &target in &input.typings {
            if seen_types.insert(target) {
                types.push(target);
            }
        }
        queue.extend(
            input
                .features
                .iter()
                .copied()
                .map(|target| (target, depth + 1)),
        );
        input_order.push(current);
        inputs.insert(current, input);
    }
    // Required paths are certified over the exact graph already used to gather
    // candidate types. No separate implied graph or recursive provider call can
    // satisfy a missing obligation. Domain applicability remains the adapter's
    // responsibility; missing paths never synthesize an edge.
    if graph_issue.is_none() && required_issue.is_none() {
        for source in input_order {
            let input = &inputs[&source];
            if let Err(issue) = required_paths(&inputs, source, input, steps, maximum) {
                required_issue = Some(issue);
                break;
            }
        }
    }
    let mut candidates = Vec::new();
    for &general in &types {
        let mut removed = false;
        let mut ambiguous = false;
        for &specific in &types {
            if specific == general {
                continue;
            }
            if !charge(steps, 1, maximum) {
                graph_issue = Some(Incomplete::Budget);
                break;
            }
            match evidence.specializes(specific, general, steps) {
                Comparison::Yes => {
                    removed = true;
                    break;
                }
                Comparison::Unknown => ambiguous = true,
                Comparison::No => {}
            }
        }
        if !removed {
            candidates.push(general);
            if ambiguous {
                graph_issue.get_or_insert(Incomplete::AmbiguousSpecialization);
            }
        }
        if *steps > maximum {
            break;
        }
    }
    // Exhaustion overrides other qualifications, including an apparent positive
    // from a provider that returned after consuming its final allowed step.
    if *steps > maximum {
        graph_issue = Some(Incomplete::Budget);
    }
    Projection {
        candidates,
        graph_issue,
        required_issue,
        steps: *steps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[derive(Default)]
    struct Graph {
        inputs: HashMap<usize, Inputs>,
        pairs: HashMap<(usize, usize), Comparison>,
        functions: HashSet<usize>,
        behaviors: HashSet<usize>,
    }
    impl Evidence for Graph {
        fn inputs(&mut self, f: usize, steps: &mut usize) -> Result<Inputs, Incomplete> {
            *steps += 1;
            self.inputs
                .get(&f)
                .cloned()
                .ok_or(Incomplete::InvalidElement)
        }
        fn specializes(&mut self, s: usize, g: usize, steps: &mut usize) -> Comparison {
            *steps += 1;
            self.pairs.get(&(s, g)).copied().unwrap_or(Comparison::No)
        }
        fn is_behavior(&self, e: usize) -> bool {
            self.behaviors.contains(&e) || self.functions.contains(&e)
        }
        fn is_function(&self, e: usize) -> bool {
            self.functions.contains(&e)
        }
    }
    fn input(typings: Vec<usize>, features: Vec<usize>) -> Inputs {
        Inputs {
            typings,
            features,
            graph_issue: None,
            required_issue: None,
            required_features: Vec::new(),
            required_typings: Vec::new(),
        }
    }
    fn run(g: &mut Graph) -> Projection {
        project(g, 0, &mut 0, 1000, 16)
    }
    #[test]
    fn closure_keeps_identity_order_handles_cycles_and_removes_only_proven_redundancy() {
        let mut g = Graph::default();
        g.inputs.insert(0, input(vec![5], vec![1, 2]));
        g.inputs.insert(1, input(vec![6], vec![0]));
        g.inputs.insert(2, input(vec![7], vec![]));
        g.functions.insert(6);
        g.pairs.insert((6, 5), Comparison::Yes);
        g.pairs.insert((6, 7), Comparison::Yes);
        let p = run(&mut g);
        assert!(p.complete());
        assert_eq!(p.candidates, vec![6]);
        assert_eq!(p.function(&g), Ok(Some(6)));
    }
    #[test]
    fn unknown_comparison_cannot_choose_an_arbitrary_function() {
        let mut g = Graph::default();
        g.inputs.insert(0, input(vec![5, 6], vec![]));
        g.functions.extend([5, 6]);
        g.pairs.insert((5, 6), Comparison::Unknown);
        assert_eq!(
            run(&mut g).function(&g),
            Err(Incomplete::AmbiguousSpecialization)
        );
        g.pairs.clear();
        assert_eq!(
            run(&mut g).function(&g),
            Err(Incomplete::InvalidFunctionMultiplicity)
        );
    }
    #[test]
    fn required_coverage_and_missing_dependency_prevent_empty_or_unique_success() {
        let mut g = Graph::default();
        let mut i = input(vec![5], vec![]);
        i.required_issue = Some(Incomplete::MissingRequiredFamilies);
        g.inputs.insert(0, i);
        g.functions.insert(5);
        assert_eq!(
            run(&mut g).function(&g),
            Err(Incomplete::MissingRequiredFamilies)
        );
        g.inputs.insert(0, input(vec![], vec![1]));
        assert_eq!(run(&mut g).function(&g), Err(Incomplete::InvalidElement));
    }
    #[test]
    fn expression_narrowing_does_not_filter_away_other_behaviors() {
        let mut g = Graph::default();
        g.inputs.insert(0, input(vec![5], vec![]));
        g.behaviors.insert(5);
        let p = run(&mut g);
        assert_eq!(p.behaviors(&g), Ok(vec![5]));
        assert_eq!(p.function(&g), Err(Incomplete::InvalidBehaviorKind));
    }
    #[test]
    fn bounded_failure_does_not_poison_fresh_retry() {
        let mut g = Graph::default();
        g.inputs.insert(0, input(vec![5], vec![]));
        g.functions.insert(5);
        let p = project(&mut g, 0, &mut 10, 10, 16);
        assert_eq!(p.function(&g), Err(Incomplete::Budget));
        assert_eq!(run(&mut g).function(&g), Ok(Some(5)));
    }
    #[test]
    fn required_paths_are_source_local_and_cannot_use_a_sibling_witness() {
        let mut g = Graph::default();
        g.inputs.insert(0, input(vec![5], vec![1, 2]));
        let mut first = input(vec![], vec![]);
        first.required_features.push(2);
        g.inputs.insert(1, first);
        g.inputs.insert(2, input(vec![], vec![]));
        g.functions.insert(5);
        assert_eq!(
            run(&mut g).function(&g),
            Err(Incomplete::MissingRequiredFamilies)
        );
        g.inputs.get_mut(&1).unwrap().features.push(2);
        assert_eq!(run(&mut g).function(&g), Ok(Some(5)));
    }
    #[test]
    fn required_typing_witness_uses_complete_shared_closure_and_retries() {
        let mut g = Graph::default();
        let mut root = input(vec![5], vec![1]);
        root.required_features = vec![0, 1];
        root.required_typings = vec![6];
        g.inputs.insert(0, root);
        g.inputs.insert(1, input(vec![], vec![0]));
        g.functions.insert(5);
        assert_eq!(
            run(&mut g).function(&g),
            Err(Incomplete::MissingRequiredFamilies)
        );
        g.inputs.get_mut(&1).unwrap().typings.push(6);
        g.pairs.insert((5, 6), Comparison::Yes);
        assert_eq!(run(&mut g).function(&g), Ok(Some(5)));
        let failed = project(&mut g, 0, &mut 0, 8, 16);
        assert_eq!(failed.function(&g), Err(Incomplete::Budget));
        assert_eq!(run(&mut g).function(&g), Ok(Some(5)));
    }
}

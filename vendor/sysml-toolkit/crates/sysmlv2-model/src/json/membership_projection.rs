//! Shared ordered Feature/membership reduction used by positional planning.
//! Inputs are provider facts; this algebra does not certify their completeness.
//! Checked effective-input consumers must validate the same provider graph before
//! treating the returned sequence as complete. No lookup, publication, or cache.
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Membership {
    pub(super) relationship: usize,
    pub(super) member: usize,
}

#[derive(PartialEq, Eq)]
pub(super) struct Node {
    pub(super) owned: Vec<usize>,
    pub(super) owned_memberships: Vec<Membership>,
    pub(super) bases: Vec<usize>,
    pub(super) public_imports: Vec<Membership>,
    pub(super) protected_imports: Vec<Membership>,
    pub(super) memberships_complete: bool,
}

/// Classification facts belong to the caller's provider. These predicates do
/// not imply that the owner's relationship or inheritance domain is complete.
pub(super) trait MembershipFacts {
    fn is_feature(&self, member: usize) -> bool;
    fn is_feature_membership(&self, relationship: usize) -> bool;
    fn is_protected(&self, relationship: usize) -> bool;
}

/// The memberships each type exports, by type.
pub(super) trait Exported {
    fn exported(&self, e: &usize) -> Option<&Vec<Membership>>;
}
impl Exported for HashMap<usize, Vec<Membership>> {
    fn exported(&self, e: &usize) -> Option<&Vec<Membership>> {
        self.get(e)
    }
}
/// A planning's own exported memberships over its prepared library's.
pub(super) struct ExportedOver<'a> {
    pub own: &'a HashMap<usize, Vec<Membership>>,
    pub library: &'a HashMap<usize, Vec<Membership>>,
}
impl Exported for ExportedOver<'_> {
    fn exported(&self, e: &usize) -> Option<&Vec<Membership>> {
        self.own.get(e).or_else(|| self.library.get(e))
    }
}

#[derive(Default)]
pub(super) struct Reduction {
    seen: HashSet<usize>,
    owned_direct: HashSet<usize>,
    counts: HashMap<usize, usize>,
    shadow: HashSet<usize>,
}
impl Reduction {
    /// Ordered union of each direct base's complete nonprivate projection.
    pub(super) fn inheritable(
        &mut self,
        node: &Node,
        exported: &dyn Exported,
        steps: &mut Option<&mut usize>,
    ) -> Option<Vec<Membership>> {
        let mut inherited = Vec::new();
        reset_scratch_set(&mut self.seen);
        for &base in &node.bases {
            let exported = exported
                .exported(&base)
                .expect("a complete base exports its memberships");
            // Admission belongs to the contributing membership or import,
            // including import-all contributions of private members.
            charge(steps, exported.len().saturating_add(1))?;
            for &membership in exported {
                if self.seen.insert(membership.relationship) {
                    inherited.push(membership);
                }
            }
        }
        Some(inherited)
    }
    pub(super) fn reduce(
        &mut self,
        node: &Node,
        exported: &dyn Exported,
        redefinitions: &dyn super::positional::DirectBases,
        traversal: &mut Reachability,
        facts: &impl MembershipFacts,
        steps: &mut Option<&mut usize>,
    ) -> Option<(Vec<usize>, Vec<Membership>)> {
        let mut inherited = self.inheritable(node, exported, steps)?;
        reset_scratch_set(&mut self.owned_direct);
        for feature in &node.owned {
            charge(
                steps,
                redefinitions
                    .get(feature)
                    .map_or(0, Vec::len)
                    .saturating_add(1),
            )?;
        }
        self.owned_direct.extend(
            node.owned
                .iter()
                .flat_map(|f| redefinitions.get(f).into_iter().flatten().copied()),
        );
        reset_scratch_map(&mut self.counts);
        charge(steps, inherited.len())?;
        for membership in &inherited {
            if facts.is_feature(membership.member) {
                *self.counts.entry(membership.member).or_insert(0usize) += 1;
            }
        }
        reset_scratch_set(&mut self.shadow);
        // Every membership of one Feature has the same closure and
        // owned-feature comparison, so traverse that Feature only once.
        for (&feature, &count) in &self.counts {
            charge(steps, 1)?;
            if redefinitions.get(&feature).is_none_or(Vec::is_empty) {
                // The closure is exactly this feature; no traversal or
                // intermediate collection is necessary.
                if count > 1 || self.owned_direct.contains(&feature) {
                    self.shadow.insert(feature);
                }
                continue;
            }
            // Reflexive closure suppresses another Membership of this
            // same Feature, but never suppresses its own Membership.
            // Consume each reached identity while walking: materializing a
            // closure and scanning it twice repeats work for every inherited
            // Feature of every Type. The graph remains fixed during this walk;
            // fresh marks still observe additions between reductions.
            let mut owned_match = false;
            traversal.for_each_reachable_budget(feature, redefinitions, steps, |target| {
                if target != feature || count > 1 {
                    self.shadow.insert(target);
                }
                owned_match |= self.owned_direct.contains(&target);
            })?;
            if owned_match {
                self.shadow.insert(feature);
            }
        }
        charge(steps, inherited.len().saturating_add(node.owned.len()))?;
        let mut features = node.owned.clone();
        // Retain membership order and collect its Feature projection together.
        inherited.retain(|membership| {
            let survives = !self.counts.contains_key(&membership.member)
                || !self.shadow.contains(&membership.member);
            if survives && facts.is_feature_membership(membership.relationship) {
                features.push(membership.member);
            }
            survives
        });
        Some((features, inherited))
    }
    pub(super) fn export(
        &mut self,
        node: &Node,
        surviving: Vec<Membership>,
        facts: &impl MembershipFacts,
        steps: &mut Option<&mut usize>,
    ) -> Option<Vec<Membership>> {
        charge(
            steps,
            node.owned_memberships
                .len()
                .saturating_mul(4)
                .saturating_add(node.public_imports.len().saturating_mul(2))
                .saturating_add(node.protected_imports.len().saturating_mul(2))
                .saturating_add(surviving.len().saturating_mul(2)),
        )?;
        let mut visible = Vec::new();
        for protected in [false, true] {
            visible.extend(
                node.owned_memberships
                    .iter()
                    .copied()
                    .filter(|m| facts.is_protected(m.relationship) == protected),
            );
            visible.extend(if protected {
                &node.protected_imports
            } else {
                &node.public_imports
            });
        }
        visible.extend(surviving);
        reset_scratch_set(&mut self.seen);
        visible.retain(|m| self.seen.insert(m.relationship));
        Some(visible)
    }
}

// A wide node must not make every later tiny node clear a wide hash table.
// Retain ordinary capacities, but discard an oversized table after that first
// wide-to-small transition; its one large clear is charged to the wide node.
pub(super) fn reset_scratch_set<T>(scratch: &mut HashSet<T>) {
    if scratch.capacity() > scratch.len().max(32).saturating_mul(4) {
        *scratch = HashSet::new();
    } else {
        scratch.clear();
    }
}

pub(super) fn reset_scratch_map<K, V>(scratch: &mut HashMap<K, V>) {
    if scratch.capacity() > scratch.len().max(32).saturating_mul(4) {
        *scratch = HashMap::new();
    } else {
        scratch.clear();
    }
}

/// Scratch space for one planning pass. The graph grows while planning, so
/// every traversal gets fresh marks; only storage is reused, never reachability.
pub(super) struct Reachability {
    pub(super) marks: Vec<usize>,
    sparse: Option<HashSet<usize>>,
    pub(super) generation: usize,
    pub(super) todo: Vec<usize>,
    pub(super) reached: Vec<usize>,
}

impl Reachability {
    pub(super) fn new(elements: usize) -> Self {
        Self {
            marks: vec![0; elements],
            sparse: None,
            generation: 0,
            todo: Vec::new(),
            reached: Vec::new(),
        }
    }

    /// Small certified slices need marks only for visited identities. This
    /// reuses the same traversal, without allocating for unrelated graph rows.
    pub(super) fn sparse() -> Self {
        Self {
            marks: Vec::new(),
            sparse: Some(HashSet::new()),
            generation: 0,
            todo: Vec::new(),
            reached: Vec::new(),
        }
    }

    fn begin(&mut self, steps: &mut Option<&mut usize>) -> Option<()> {
        if let Some(marks) = &mut self.sparse {
            charge(steps, marks.capacity())?;
            reset_scratch_set(marks);
        } else {
            if self.generation == usize::MAX {
                charge(steps, self.marks.len())?;
                self.marks.fill(0);
                self.generation = 0;
            }
            self.generation += 1;
        }
        Some(())
    }

    fn visit(&mut self, element: usize) -> bool {
        if let Some(marks) = &mut self.sparse {
            marks.insert(element)
        } else if self.marks[element] == self.generation {
            false
        } else {
            self.marks[element] = self.generation;
            true
        }
    }

    #[cfg(test)]
    pub(super) fn reachable(
        &mut self,
        source: usize,
        graph: &HashMap<usize, Vec<usize>>,
    ) -> &[usize] {
        self.reachable_budget(source, graph, &mut None)
            .expect("unbounded traversal cannot exhaust its budget")
    }

    /// Existential reachability does not need to allocate a complete closure.
    /// Fresh marks observe graph growth between calls; no answer is cached.
    pub(super) fn reaches_budget(
        &mut self,
        source: usize,
        target: usize,
        graph: &dyn super::positional::DirectBases,
        steps: &mut Option<&mut usize>,
    ) -> Option<bool> {
        charge(steps, 1)?;
        self.begin(steps)?;
        self.todo.clear();
        self.todo.push(source);
        while let Some(e) = self.todo.pop() {
            charge(steps, 1)?;
            if e == target {
                return Some(true);
            }
            if !self.visit(e) {
                continue;
            }
            for &next in graph.get(&e).into_iter().flatten() {
                charge(steps, 1)?;
                if next == target {
                    return Some(true);
                }
                self.todo.push(next);
            }
        }
        Some(false)
    }

    pub(super) fn reachable_budget(
        &mut self,
        source: usize,
        graph: &dyn super::positional::DirectBases,
        steps: &mut Option<&mut usize>,
    ) -> Option<&[usize]> {
        let mut reached = std::mem::take(&mut self.reached);
        reached.clear();
        let result = self.for_each_reachable_budget(source, graph, steps, |e| reached.push(e));
        self.reached = reached;
        result?;
        Some(&self.reached)
    }

    /// Visit a reflexive closure once, without an intermediate collection.
    /// Each bounded visit includes its constant-time consumer. No graph answer
    /// survives this call; only traversal scratch storage is retained.
    fn for_each_reachable_budget(
        &mut self,
        source: usize,
        graph: &dyn super::positional::DirectBases,
        steps: &mut Option<&mut usize>,
        mut visit: impl FnMut(usize),
    ) -> Option<()> {
        charge(steps, 1)?;
        self.begin(steps)?;
        self.todo.clear();
        self.todo.push(source);
        while let Some(e) = self.todo.pop() {
            charge(steps, 1)?;
            if self.visit(e) {
                visit(e);
                charge(steps, graph.get(&e).map_or(0, Vec::len))?;
                self.todo
                    .extend(graph.get(&e).into_iter().flatten().copied());
            }
        }
        Some(())
    }
}

pub(super) fn charge(steps: &mut Option<&mut usize>, amount: usize) -> Option<()> {
    if let Some(steps) = steps {
        **steps = steps.saturating_add(amount);
        if **steps > crate::eval::MAX_STEPS {
            return None;
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Facts;
    impl MembershipFacts for Facts {
        fn is_feature(&self, member: usize) -> bool {
            member < 12
        }
        fn is_feature_membership(&self, relationship: usize) -> bool {
            relationship % 3 != 0
        }
        fn is_protected(&self, relationship: usize) -> bool {
            relationship % 2 == 0
        }
    }

    /// Deliberately pairwise oracle: apply the membership suppression rule to
    /// each identity independently, without the production shadow-set algebra.
    fn reference_reduction(
        node: &Node,
        exported: &HashMap<usize, Vec<Membership>>,
        graph: &HashMap<usize, Vec<usize>>,
    ) -> (Vec<usize>, Vec<Membership>) {
        let closure = |source| {
            let mut reached = HashSet::new();
            let mut todo = vec![source];
            while let Some(at) = todo.pop() {
                if reached.insert(at) {
                    todo.extend(graph.get(&at).into_iter().flatten().copied());
                }
            }
            reached
        };
        let mut inherited = Vec::new();
        for base in &node.bases {
            for &membership in &exported[base] {
                if !inherited
                    .iter()
                    .any(|m: &Membership| m.relationship == membership.relationship)
                {
                    inherited.push(membership);
                }
            }
        }
        let surviving: Vec<_> = inherited
            .iter()
            .copied()
            .filter(|m| {
                if !Facts.is_feature(m.member) {
                    return true;
                }
                let own_closure = closure(m.member);
                !inherited.iter().any(|other| {
                    other.relationship != m.relationship
                        && Facts.is_feature(other.member)
                        && closure(other.member).contains(&m.member)
                }) && !node.owned.iter().any(|owned| {
                    graph
                        .get(owned)
                        .into_iter()
                        .flatten()
                        .any(|target| own_closure.contains(target))
                })
            })
            .collect();
        let mut features = node.owned.clone();
        features.extend(surviving.iter().filter_map(|m| {
            Facts
                .is_feature_membership(m.relationship)
                .then_some(m.member)
        }));
        (features, surviving)
    }

    #[test]
    fn streamed_reduction_matches_pairwise_rules_through_graph_changes_and_budget_retry() {
        let mut graph: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut reduction = Reduction::default();
        let mut dense = Reachability::new(16);
        let mut sparse = Reachability::sparse();
        for round in 0..96 {
            // Repeated edges, cycles, self-edges and removals must never reuse
            // an earlier closure. Distinct memberships of one Feature remain
            // distinct until the reflexive suppression rule removes them.
            let edges = graph.entry(round % 12).or_default();
            if round % 5 == 0 {
                edges.clear();
            }
            edges.push((round * 7 + round / 12) % 12);
            let mut exported = HashMap::new();
            for base in 0..3 {
                exported.insert(
                    base,
                    (0..24)
                        .filter(|i| (i + base + round) % 4 != 0)
                        .map(|i| Membership {
                            relationship: 100 + i,
                            member: (i + round) % 16,
                        })
                        .collect(),
                );
            }
            let node = Node {
                owned: if round % 3 == 0 {
                    vec![]
                } else {
                    vec![round % 12, (round * 5 + 3) % 12]
                },
                owned_memberships: vec![],
                bases: vec![round % 3, (round + 1) % 3, (round + 2) % 3],
                public_imports: vec![],
                protected_imports: vec![],
                memberships_complete: true,
            };
            let expected = reference_reduction(&node, &exported, &graph);
            for traversal in [&mut dense, &mut sparse] {
                let mut exhausted = crate::eval::MAX_STEPS - 100;
                assert!(
                    reduction
                        .reduce(
                            &node,
                            &exported,
                            &graph,
                            traversal,
                            &Facts,
                            &mut Some(&mut exhausted),
                        )
                        .is_none()
                );
                let (features, memberships) = reduction
                    .reduce(
                        &node,
                        &exported,
                        &graph,
                        traversal,
                        &Facts,
                        &mut Some(&mut 0),
                    )
                    .unwrap();
                assert_eq!(features, expected.0, "round {round}");
                assert!(memberships == expected.1, "round {round}");
            }
        }
    }

    #[test]
    fn sparse_wide_to_small_traversal_work_does_not_retain_wide_clear_cost() {
        let graph = HashMap::from([(0, (1..10_000).collect()), (10_000, vec![10_001])]);
        let mut dense = Reachability::new(10_002);
        let mut sparse = Reachability::sparse();
        let mut steps = 0;
        assert_eq!(
            sparse
                .reachable_budget(0, &graph, &mut Some(&mut steps))
                .unwrap(),
            dense.reachable(0, &graph)
        );
        for _ in 0..2 {
            assert_eq!(
                sparse
                    .reachable_budget(10_000, &graph, &mut Some(&mut steps))
                    .unwrap(),
                dense.reachable(10_000, &graph)
            );
        }
        let small_start = steps;
        for _ in 0..1_000 {
            assert_eq!(
                sparse
                    .reachable_budget(10_000, &graph, &mut Some(&mut steps))
                    .unwrap(),
                &[10_000, 10_001]
            );
        }
        assert!(steps - small_start < 10_000);
        let mut exhausted = crate::eval::MAX_STEPS;
        assert!(
            sparse
                .reachable_budget(0, &graph, &mut Some(&mut exhausted))
                .is_none()
        );
        assert_eq!(sparse.reachable(0, &graph), dense.reachable(0, &graph));
    }
    #[test]
    fn existential_reachability_matches_complete_closure_through_graph_growth() {
        let mut graph: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut traversal = Reachability::new(12);
        let mut sparse = Reachability::sparse();
        for round in 0..36 {
            graph
                .entry(round % 12)
                .or_default()
                .push((round * 7 + 3 + round / 12) % 12);
            for source in 0..12 {
                let expected = traversal.reachable(source, &graph).to_vec();
                assert_eq!(sparse.reachable(source, &graph), expected);
                for target in 0..12 {
                    assert_eq!(
                        traversal.reaches_budget(source, target, &graph, &mut Some(&mut 0)),
                        Some(expected.contains(&target))
                    );
                    assert_eq!(
                        sparse.reaches_budget(source, target, &graph, &mut Some(&mut 0)),
                        Some(expected.contains(&target))
                    );
                }
            }
        }
        traversal.generation = usize::MAX;
        let mut exhausted = crate::eval::MAX_STEPS;
        assert!(
            traversal
                .reaches_budget(0, 3, &graph, &mut Some(&mut exhausted))
                .is_none()
        );
        assert_eq!(
            traversal.reaches_budget(0, 3, &graph, &mut Some(&mut 0)),
            Some(true)
        );
    }
}

use std::collections::HashSet;
use std::hash::Hash;

use sysmlv2_model::json::{ElementRef, ResolvedModel, UnresolvedReference};

/// Index unresolved reference carriers and their owning subtrees once per
/// analysis. An end's own subtree includes synthesized feature-chain links and
/// flow redefinitions, but not its siblings or the connection's other members.
/// Do not follow semantic reference targets: those are not ownership edges.
pub(crate) fn unresolved_subtrees(
    graph: &mut ResolvedModel,
    references: &[UnresolvedReference],
) -> HashSet<ElementRef> {
    ancestor_closure(references.iter().map(|site| site.owner), |e| graph.owner(e))
}

fn ancestor_closure<T: Copy + Eq + Hash>(
    starts: impl IntoIterator<Item = T>,
    mut owner: impl FnMut(T) -> Option<T>,
) -> HashSet<T> {
    let mut marked = HashSet::new();
    for start in starts {
        let mut current = Some(start);
        while let Some(element) = current {
            // An earlier walk already marked this node and all its ancestors.
            // This also terminates if a provider ever returns cyclic ownership.
            if !marked.insert(element) {
                break;
            }
            current = owner(element);
        }
    }
    marked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_ancestry_is_visited_once_without_marking_siblings() {
        // 10,000 unresolved carriers share one deep ownership spine. The
        // number of owner queries must grow with distinct nodes, not walks.
        let mut visits = 0;
        let marked = ancestor_closure(10_000..20_000, |element| {
            visits += 1;
            match element {
                0 => None,
                10_000.. => Some(9_999),
                n => Some(n - 1),
            }
        });
        assert_eq!(marked.len(), 20_000);
        assert_eq!(visits, 20_000);
        assert!(!marked.contains(&20_000));
    }

    #[test]
    fn empty_and_cyclic_ownership_terminate_without_losing_carriers() {
        assert!(ancestor_closure(std::iter::empty::<u32>(), |_| panic!()).is_empty());
        let marked = ancestor_closure([0, 3, 0], |element| match element {
            0 => Some(1),
            1 => Some(2),
            2 => Some(0),
            _ => None,
        });
        assert_eq!(marked, HashSet::from([0, 1, 2, 3]));
    }
}

use crate::{ImportIssue, Visibility};
use sysmlv2_model::json::{DerivedValue, ElementRef, OperationError, Reference, ResolvedModel};

/// Limit contextual visibility calls across the snapshot. Each checked upstream
/// call also has its own traversal budget. Exhaustion discards the whole closure.
pub(crate) const VISIBILITY_QUERY_LIMIT: usize = 4096;

pub(crate) fn imported_memberships(
    graph: &mut ResolvedModel,
    owner: ElementRef,
    remaining: &mut usize,
) -> Result<Vec<(ElementRef, Visibility)>, ImportIssue> {
    let result = graph
        .invoke_operation(
            owner,
            "Root-Namespaces-Namespace-importedMemberships_Namespace",
            &[DerivedValue::Elements(vec![])],
        )
        .map_err(issue)?;
    let DerivedValue::References(memberships) = result.value else {
        return Err(ImportIssue::IncompleteProvider);
    };
    if memberships.len() > *remaining {
        return Err(ImportIssue::WorkLimit);
    }
    let mut bindings = Vec::with_capacity(memberships.len());
    for membership in memberships {
        let Reference::Element(membership) = membership else {
            return Err(ImportIssue::IncompleteProvider);
        };
        *remaining -= 1;
        let visibility = graph
            .invoke_operation(
                owner,
                "Root-Namespaces-Namespace-visibilityOf_Membership",
                &[DerivedValue::Element(membership)],
            )
            .map_err(issue)?;
        let visibility = match visibility.value {
            DerivedValue::Str(value) if value == "public" => Visibility::Public,
            DerivedValue::Str(value) if value == "protected" => Visibility::Protected,
            DerivedValue::Str(value) if value == "private" => Visibility::Private,
            _ => return Err(ImportIssue::IncompleteProvider),
        };
        bindings.push((membership, visibility));
    }
    Ok(bindings)
}

fn issue(error: OperationError) -> ImportIssue {
    match error {
        OperationError::WorkLimit { .. } => ImportIssue::WorkLimit,
        OperationError::Unsupported { .. } | OperationError::UnsupportedConfiguration { .. } => {
            ImportIssue::Unsupported
        }
        _ => ImportIssue::IncompleteProvider,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysmlv2_model::model::Model;

    #[test]
    fn visibility_budget_never_returns_a_partial_closure() {
        let mut model = Model::new();
        model.add_source(
            "imports.kerml",
            "package P { class A; class B; } package Q { private import P::*; }",
        );
        let mut graph = ResolvedModel::build(&model);
        let q = graph.resolve_qualified("Q").unwrap();
        assert_eq!(
            imported_memberships(&mut graph, q, &mut 1),
            Err(ImportIssue::WorkLimit)
        );
        assert_eq!(
            imported_memberships(&mut graph, q, &mut 2).unwrap().len(),
            2
        );
    }
}

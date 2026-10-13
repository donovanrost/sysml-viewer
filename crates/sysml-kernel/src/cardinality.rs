use crate::{Cardinality, CardinalityIssue};
use sysmlv2_model::{
    check,
    json::{CardinalityIssue as UpstreamIssue, ElementRef, ResolvedModel},
};

/// Use the checked constraint reader, never the approximate f64 compatibility
/// accessor. In particular an unavailable provider is not an unconstrained type.
pub(crate) fn cardinality(graph: &mut ResolvedModel, element: ElementRef) -> Cardinality {
    if !check::metaclass_conforms(graph.element_type(element), "Type") {
        return Cardinality::NotApplicable;
    }
    match graph.cardinality_report(element).bounds {
        Ok(bounds) => Cardinality::Evaluated {
            lower: bounds.lower.to_string(),
            upper: bounds.upper.map(|n| n.to_string()),
        },
        Err(issue) => Cardinality::Unknown {
            reason: match issue {
                UpstreamIssue::InvalidRelationship => CardinalityIssue::InvalidRelationship,
                UpstreamIssue::IncompleteProvider => CardinalityIssue::IncompleteProvider,
                UpstreamIssue::UnsupportedBound => CardinalityIssue::UnsupportedBound,
                UpstreamIssue::CyclicDependency => CardinalityIssue::CyclicDependency,
                UpstreamIssue::ConflictingConstraints => CardinalityIssue::ConflictingConstraints,
                UpstreamIssue::WorkLimit => CardinalityIssue::WorkLimit,
                _ => CardinalityIssue::Unavailable,
            },
        },
    }
}

pub(crate) fn display(bounds: &Cardinality) -> Option<String> {
    let Cardinality::Evaluated { lower, upper } = bounds else {
        return None;
    };
    let high = upper.as_deref().unwrap_or("*");
    Some(if upper.as_ref() == Some(lower) {
        format!("[{lower}]")
    } else {
        format!("[{lower}..{high}]")
    })
}

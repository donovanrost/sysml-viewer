//! Feature-value scalar conformance: a bound value whose evaluated
//! scalar partition can never inhabit any of the feature's declared
//! types.
use crate::{json::ResolvedModel, model::Model};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::OnceLock;
use sysmlv2_syntax::diag::Diagnostic;
use uuid::Uuid;

pub(super) fn validate(r: &mut ResolvedModel, model: &Model) -> Vec<(usize, Diagnostic)> {
    let mut out = Vec::new();
    // --- feature-value scalar conformance ---
    // Narrowed to what is provable. A stricter static rule — the value
    // expression's *declared* result type must conform to the feature's
    // declared type — would reject 15 official corpus files whose bindings
    // are legal under KerML multiple classification: a `Rational`-typed
    // literal's value may well inhabit a sibling subtype of `Real`
    // (`ScalarValues::Rational does not conform to EnumerationTest::Size`
    // is exactly the corpus's enum-restriction idiom, and the ISO-8601
    // string encodings and metadata levels follow the same shape). What
    // *is* provable is kind-level disjointness of the evaluated value:
    // `ScalarValues` partitions its scalars into Boolean, String, and the
    // Number tower, and a value of one partition is never an instance of
    // a type in another; within the tower, a non-integral rational is
    // never an instance of an `Integer`-conforming type. Only loaded
    // standard-library identities establish scalar restrictions (KerML 9.1).
    // Every explicit typing/specialization restriction applies, including
    // those inherited through subsetting/redefinition: Feature::type is an
    // intersection, not a union (KerML 8.3.3.3). Unknown restrictions cannot
    // cancel a known incompatibility. Unevaluable values, quantities,
    // sequences, and element values stay silent. Warnings, per checker policy.
    let value_sites: Vec<(usize, usize, sysmlv2_syntax::ast::Expr)> =
        r.b.values
            .iter()
            .filter(|(o, _)| super::user_owned(&r.b, model, **o))
            .map(|(o, (s, e))| (*o, *s, e.clone()))
            .collect();
    for (owner, scope, expr) in value_sites {
        let unit = r.b.unit_of_elem(owner);
        if model.is_library_unit(unit) {
            continue;
        }
        let Some(constraints) = scalar_constraints(&mut r.b, owner) else {
            continue;
        };
        if constraints.is_empty() {
            continue;
        }
        let origin = r.b.set_identity_origin(owner);
        let value = crate::eval::evaluate_expr_in(&mut r.b, scope, &expr);
        r.b.identity_origin_unit = origin;
        let Ok(v) = value else {
            continue; // undecided, not wrong
        };
        let Some(value_kind) = value_scalar_kind(&v) else {
            continue; // quantities, sequences, instances, elements
        };
        let Some((ty, kind)) = constraints
            .into_iter()
            .find(|(_, kind)| !scalar_kind_admits(*kind, value_kind))
        else {
            continue;
        };
        let ty_name = r.b.elements[ty]
            .props
            .get("declaredName")
            .and_then(|v| v.as_str())
            .unwrap_or("<anonymous>")
            .to_string();
        let msg = match value_kind {
            ScalarKind::Number { integral: false }
                if matches!(kind, ScalarKind::Number { integral: true }) =>
            {
                format!(
                    "feature value {v} is not an integer and can never conform \
                     to the required type `{ty_name}`"
                )
            }
            _ => {
                let vdesc = match value_kind {
                    ScalarKind::Boolean => "a Boolean",
                    ScalarKind::Str => "a String",
                    ScalarKind::Number { .. } => "a number",
                };
                format!(
                    "feature value evaluates to {vdesc}, which can never \
                     conform to the required type `{ty_name}`"
                )
            }
        };
        out.push((unit, Diagnostic::warning(expr.span, msg)));
    }
    out
}

/// The provably-disjoint `ScalarValues` partitions: Boolean, String, and
/// the Number tower (with `Integer`-and-below tracked for integrality).
#[derive(Clone, Copy, PartialEq)]
enum ScalarKind {
    Boolean,
    Str,
    Number { integral: bool },
}

/// The partition an evaluated value inhabits — `None` for the values
/// the conformance check stays silent on (quantities, sequences,
/// instances, elements).
fn value_scalar_kind(v: &crate::eval::Value) -> Option<ScalarKind> {
    use crate::eval::Value;
    Some(match v {
        Value::Boolean(_) => ScalarKind::Boolean,
        Value::String(_) => ScalarKind::Str,
        Value::Integer(_) => ScalarKind::Number { integral: true },
        Value::Rational(r) => ScalarKind::Number {
            integral: r.is_integer(),
        },
        Value::Real(f) => ScalarKind::Number {
            integral: f.fract() == 0.0 && f.is_finite(),
        },
        _ => return None,
    })
}

/// Whether a declared partition admits a value's: same partition, and
/// within the Number tower an integral requirement met by an integral
/// value. Cross-partition pairs are provably disjoint.
fn scalar_kind_admits(decl: ScalarKind, value: ScalarKind) -> bool {
    match (decl, value) {
        (ScalarKind::Boolean, ScalarKind::Boolean) => true,
        (ScalarKind::Str, ScalarKind::Str) => true,
        (ScalarKind::Number { integral: need }, ScalarKind::Number { integral: have }) => {
            !need || have
        }
        _ => false,
    }
}

impl crate::json::ResolvedModel {
    /// The feature-value scalar-conformance verdict the semantic check
    /// would reach for `value` bound to a feature declaring exactly
    /// `declared` as its types: `Some(false)` when the check would
    /// report the value as never conforming to any required scalar restriction,
    /// `Some(true)` when every declared type has known scalar restrictions and
    /// all admit it, `None` when classification is incomplete and no known
    /// restriction disproves it (or the value is not scalar). This checks only
    /// scalar partitions/integrality, not arbitrary subtype constraints.
    pub fn scalar_value_admitted(
        &mut self,
        declared: &[crate::json::ElementRef],
        value: &crate::eval::Value,
    ) -> Option<bool> {
        let value_kind = value_scalar_kind(value)?;
        let mut complete = !declared.is_empty();
        for ty in declared {
            let Some(constraints) = scalar_constraints(&mut self.b, ty.0) else {
                complete = false;
                continue;
            };
            if constraints.is_empty() {
                complete = false;
            }
            if constraints
                .iter()
                .any(|(_, k)| !scalar_kind_admits(*k, value_kind))
            {
                return Some(false);
            }
        }
        complete.then_some(true)
    }
}

/// Collect every known restriction in the explicit supertype closure. A wider
/// ancestor must not hide a narrower one or make results depend on base order.
/// No matching identity means unknown, not compatibility; an exhausted walk
/// returns no partial certificate. Cycles are visited once.
fn scalar_constraints(b: &mut crate::json::Builder, ty: usize) -> Option<Vec<(usize, ScalarKind)>> {
    let mut queue: VecDeque<usize> = VecDeque::new();
    queue.push_back(ty);
    let mut seen: HashSet<usize> = HashSet::new();
    let mut constraints = Vec::new();
    while let Some(t) = queue.pop_front() {
        if !seen.insert(t) {
            continue;
        }
        if seen.len() > 64 {
            return None;
        }
        if t < b.lib_boundary {
            if let Some(kind) = scalar_identities().get(&b.elem_id(t)) {
                constraints.push((t, *kind));
            }
        }
        for s in b.explicit_supertype_elems(t) {
            queue.push_back(s);
        }
    }
    Some(constraints)
}

/// Same loaded-library identity discipline as evaluator intrinsic dispatch.
/// User-owned names, even `ScalarValues::Integer`, never designate primitives.
fn scalar_identities() -> &'static HashMap<Uuid, ScalarKind> {
    static IDS: OnceLock<HashMap<Uuid, ScalarKind>> = OnceLock::new();
    IDS.get_or_init(|| {
        let namespace = Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            b"https://www.omg.org/spec/KerML/ScalarValues",
        );
        [
            ("Boolean", ScalarKind::Boolean),
            ("String", ScalarKind::Str),
            ("Integer", ScalarKind::Number { integral: true }),
            ("Natural", ScalarKind::Number { integral: true }),
            ("Positive", ScalarKind::Number { integral: true }),
            ("NumericalValue", ScalarKind::Number { integral: false }),
            ("Number", ScalarKind::Number { integral: false }),
            ("Complex", ScalarKind::Number { integral: false }),
            ("Real", ScalarKind::Number { integral: false }),
            ("Rational", ScalarKind::Number { integral: false }),
        ]
        .into_iter()
        .map(|(name, kind)| {
            (
                Uuid::new_v5(&namespace, format!("ScalarValues::{name}").as_bytes()),
                kind,
            )
        })
        .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn scalar_checks_restore_active_source_identity_on_every_outcome() {
        let mut model = Model::new();
        model.add_library_source(
            "scalars.kerml",
            "standard library package ScalarValues { datatype Integer; }",
        );
        model.add_source("first.kerml", "package A;");
        model.add_source("second.kerml", "package B { datatype Integer specializes ScalarValues::Integer; feature n = 2; feature bound : Integer = '88888888-8888-4888-8888-888888888888'; feature invalid : Integer = true; feature unknown : Integer = missing; feature unbound; feature unconstrained = 3; feature empty : Integer = (); }");
        assert!(!model.has_errors());
        let mut r = ResolvedModel::build(&model);
        let n = r.resolve_qualified("B::n").unwrap();
        let id = "88888888-8888-4888-8888-888888888888".parse().unwrap();
        r.override_ids(&HashMap::from([(r.element_id(n), id)]));
        assert!(r.bind_id_spelled_references().contains(&id));
        let prior = r.b.set_identity_origin(n.0);
        assert_eq!(r.b.identity_origin_unit, Some(2));
        r.b.identity_origin_unit = prior;
        for origin in [None, Some(0)] {
            r.b.identity_origin_unit = origin;
            let findings = validate(&mut r, &model);
            assert_eq!(findings.len(), 1, "{findings:?}");
            assert_eq!(r.b.identity_origin_unit, origin);
        }
    }
}

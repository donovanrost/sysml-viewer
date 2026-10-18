//! Static ownership, specialization, and feature-property constraints.
use super::facts::{Facts, flag, is};
use crate::{
    json::{Builder, ElementRef, ResolvedModel},
    model::Model,
};
use sysmlv2_syntax::{Span, diag::Diagnostic};

fn occurrence(b: &Builder, g: &Facts, e: usize) -> bool {
    is(b, e, "Class")
        || is(b, e, "OccurrenceUsage")
        || g.typed(b, e).iter().any(|&t| is(b, t, "Class"))
}
pub(super) fn variable(b: &Builder, g: &Facts, e: usize) -> bool {
    if flag(b, e, "isVariable") {
        return true;
    }
    if !is(b, e, "Usage") {
        return false;
    }
    // SysML derives variability from an occurrence owning type. A portion
    // is a time slice/snapshot rather than a variable feature.
    !flag(b, e, "isPortion")
        && !b.elements[e]
            .props
            .get("portionKind")
            .is_some_and(|p| p.is_string())
        && g.featuring(b, e).is_some_and(|o| occurrence(b, g, o))
}
pub(super) fn validate(r: &ResolvedModel, model: &Model, g: &Facts) -> Vec<(usize, Diagnostic)> {
    let b = &r.b;
    let mut out = Vec::new();
    let mut report = |e: usize, span: Span, rule: &'static str, msg: &str| {
        let unit = b.unit_of_elem(e);
        if !model.is_library_unit(unit) {
            out.push((unit, super::rule_error(span, rule, msg)));
        }
    };
    // Every finding below is on the specializing element, so a library
    // one's rows are passed over before any test.
    for (i, (e, kind, _, qn)) in b.spec_targets.iter().enumerate() {
        if *e < b.lib_boundary
            || !matches!(*kind, "Subclassification" | "Subsetting")
            || !is(b, *e, "Classifier")
        {
            continue;
        }
        let Some(t) = b.spec_resolved.get(i).copied().flatten() else {
            continue;
        };
        let cases = [
            (
                is(b, *e, "DataType") && (is(b, t, "Class") || is(b, t, "Association")),
                "validateDataTypeSpecialization",
                "A data type cannot specialize a class or association",
            ),
            (
                is(b, *e, "Class")
                    && (is(b, t, "DataType")
                        || (is(b, t, "Association") && !is(b, *e, "Association"))),
                "validateClassSpecialization",
                "A class cannot specialize a data type or an unrelated association family",
            ),
            (
                is(b, *e, "Structure") && is(b, t, "Behavior") && !is(b, t, "Structure"),
                "validateStructureSpecialization",
                "A structure cannot specialize a behavior",
            ),
            (
                is(b, *e, "Behavior") && is(b, t, "Structure") && !is(b, t, "Behavior"),
                "validateBehaviorSpecialization",
                "A behavior cannot specialize a structure",
            ),
        ];
        for (bad, rule, msg) in cases {
            if bad {
                report(*e, qn.span, rule, msg);
            }
        }
    }
    for (i, (e, kind, _, qn)) in b.spec_targets.iter().enumerate() {
        if *e < b.lib_boundary {
            continue;
        }
        let Some(t) = b.spec_resolved.get(i).copied().flatten() else {
            continue;
        };
        if !matches!(*kind, "Subsetting" | "Redefinition") {
            continue;
        }
        if *kind == "Redefinition" {
            if g.featuring(b, *e).is_some() && g.featuring(b, *e) == g.featuring(b, t) {
                report(
                    *e,
                    qn.span,
                    "validateRedefinitionFeaturingTypes",
                    "Redefining and redefined features cannot have the same owning type",
                );
            }
            if flag(b, t, "isEnd") && !flag(b, *e, "isEnd") {
                report(
                    *e,
                    qn.span,
                    "validateRedefinitionEndConformance",
                    "A feature redefining an end must itself be an end",
                );
            }
            if g.featuring(b, *e).is_none() && g.featuring(b, t).is_none() {
                report(
                    *e,
                    qn.span,
                    "validateRedefinitionFeaturingTypes",
                    "A package-level feature cannot redefine another package-level feature",
                );
            }
        }
        if flag(b, *e, "isVariable") && !flag(b, *e, "isConstant") && flag(b, t, "isConstant") {
            report(
                *e,
                qn.span,
                "validateSubsettingConstantConformance",
                "A variable subset of a constant feature must be constant",
            );
        }
        if b.elements[*e]
            .props
            .get("isUnique")
            .and_then(|v| v.as_bool())
            == Some(false)
            && flag(b, t, "isUnique")
        {
            report(
                *e,
                qn.span,
                "validateSubsettingUniquenessConformance",
                "A subset of a unique feature must be unique",
            );
        }
    }
    // Owners of result expressions, indexed once for the per-type walk.
    let result_owners: std::collections::HashSet<usize> =
        b.result_exprs.iter().map(|(o, _, _)| *o).collect();
    for e in b.lib_boundary..b.explicit_len() {
        if model.is_library_unit(b.unit_of_elem(e)) {
            continue;
        }
        let span = g.span(b, e);
        let ty = b.elements[e].ty;
        let variation = flag(b, e, "isVariation") || is(b, e, "EnumerationDefinition");
        if variation {
            for &m in &g.members[e] {
                if is(b, m, "Usage")
                    && !is(b, m, "MetadataUsage")
                    && !is(b, m, "EnumerationUsage")
                    && b.elements[m].owning_relationship.is_some_and(|rel| {
                        is(b, rel, "FeatureMembership") && !is(b, rel, "VariantMembership")
                    })
                {
                    report(
                        m,
                        g.span(b, m),
                        if is(b, e, "Definition") {
                            "validateDefinitionVariationMembership"
                        } else {
                            "validateUsageVariationMembership"
                        },
                        "A usage owned by a variation must be a variant",
                    );
                }
            }
            if g.supers[e]
                .iter()
                .any(|&s| flag(b, s, "isVariation") || is(b, s, "EnumerationDefinition"))
            {
                report(
                    e,
                    span,
                    if is(b, e, "Definition") {
                        "validateDefinitionVariationSpecialization"
                    } else {
                        "validateUsageVariationSpecialization"
                    },
                    "A variation cannot specialize another variation",
                );
            }
        }
        for (kind, membership, rule) in [
            (
                "CaseDefinition",
                "ObjectiveMembership",
                "validateCaseDefinitionOnlyOneObjective",
            ),
            (
                "CaseUsage",
                "ObjectiveMembership",
                "validateCaseUsageOnlyOneObjective",
            ),
            (
                "RequirementDefinition",
                "SubjectMembership",
                "validateRequirementDefinitionOnlyOneSubject",
            ),
            (
                "RequirementUsage",
                "SubjectMembership",
                "validateRequirementUsageOnlyOneSubject",
            ),
        ] {
            if !is(b, e, kind) {
                continue;
            }
            let owns_role = g.members[e].iter().any(|&m| {
                b.elements[m]
                    .owning_relationship
                    .is_some_and(|rel| is(b, rel, membership))
            });
            if !owns_role
                && g.effective_members(b, e)
                    .iter()
                    .filter(|&&m| {
                        b.elements[m]
                            .owning_relationship
                            .is_some_and(|rel| is(b, rel, membership))
                    })
                    .count()
                    > 1
            {
                report(
                    e,
                    span,
                    rule,
                    "At most one effective member with this parameter role is allowed",
                );
            }
        }
        for (kind, key, one, self_rule) in [
            (
                "Unioning",
                "unioningType",
                "validateOwnedUnioningNotOne",
                "validateTypeUnioningTypesNotSelf",
            ),
            (
                "Intersecting",
                "intersectingType",
                "validateOwnedIntersectingNotOne",
                "validateTypeIntersectingTypesNotSelf",
            ),
            (
                "Differencing",
                "differencingType",
                "validateOwnedDifferencingNotOne",
                "validateTypeDifferencingTypesNotSelf",
            ),
        ] {
            let targets = g.relations(b, e, kind, key);
            if targets.len() == 1 {
                report(
                    e,
                    span,
                    one,
                    "A type operation must have zero or at least two operands",
                );
            }
            if targets.contains(&e) {
                report(
                    e,
                    span,
                    self_rule,
                    "A type operation cannot name its own type as an operand",
                );
            }
        }
        if ty == "LibraryPackage" && flag(b, e, "isStandard") {
            report(
                e,
                span,
                "validateLibraryPackageNotStandard",
                "A user library package must not be marked standard",
            );
        }
        if is(b, e, "Feature") {
            let is_var = variable(b, g, e);
            if flag(b, e, "isVariable") && !g.featuring(b, e).is_some_and(|o| occurrence(b, g, o)) {
                report(
                    e,
                    span,
                    "validateFeatureIsVariable",
                    "A variable feature must be owned by an occurrence type",
                );
            }
            if flag(b, e, "isVariable") && flag(b, e, "isPortion") {
                report(
                    e,
                    span,
                    "validateFeaturePortionNotVariable",
                    "A portion cannot be variable",
                );
            }
            if flag(b, e, "isConstant") && !is_var {
                report(
                    e,
                    span,
                    "validateFeatureConstantIsVariable",
                    "Only a variable feature can be constant",
                );
            }
            for &rel in &b.elements[e].owned_relationships {
                if b.elements[rel].ty == "FeatureValue" && flag(b, rel, "isInitial") && !is_var {
                    report(
                        e,
                        span,
                        "validateFeatureValueIsInitial",
                        "An initialized feature must be variable",
                    );
                }
            }
        }
        if is(b, e, "OccurrenceUsage") {
            let typed = g.typed(b, e);
            let n = typed
                .iter()
                .filter(|&&t| flag(b, t, "isIndividual") && is(b, t, "Definition"))
                .count();
            if n > 1 {
                report(
                    e,
                    span,
                    "validateOccurrenceUsageIndividualDefinition",
                    "At most one individual definition may type an occurrence",
                );
            }
            if flag(b, e, "isIndividual") && n != 1 && !typed.is_empty() {
                report(
                    e,
                    span,
                    "validateOccurrenceUsageIndividualUsage",
                    "An individual usage must be typed by one individual definition",
                );
            }
            if b.elements[e]
                .props
                .get("portionKind")
                .is_some_and(|p| p.is_string())
                && !g.featuring(b, e).is_some_and(|o| {
                    is(b, o, "OccurrenceDefinition") || is(b, o, "OccurrenceUsage")
                })
            {
                report(
                    e,
                    span,
                    "validateOccurrenceUsageIsPortion",
                    "A snapshot or timeslice must be owned by an occurrence definition or usage",
                );
            }
        }
        if let Some(owner) = g.owner[e] {
            if is(b, e, "Usage") && !is(b, e, "PortUsage") && flag(b, e, "isComposite") {
                let rule = match b.elements[owner].ty {
                    "PortDefinition" => Some("validatePortDefinitionOwnedUsagesNotComposite"),
                    "PortUsage" => Some("validatePortUsageNestedUsagesNotComposite"),
                    _ => None,
                };
                if let Some(rule) = rule {
                    report(
                        e,
                        span,
                        rule,
                        "A non-port usage owned by a port must be referential",
                    );
                }
            }
            if is(b, e, "PortUsage")
                && (is(b, owner, "PortDefinition") || is(b, owner, "PortUsage"))
                && g.owner[owner]
                    .is_some_and(|o| is(b, o, "PortDefinition") || is(b, o, "PortUsage"))
                && b.elements[e]
                    .owning_relationship
                    .is_some_and(|rel| b.elements[rel].ty == "VariantMembership")
                && flag(b, e, "isComposite")
            {
                report(
                    e,
                    span,
                    "validatePortUsageIsReference",
                    "A port nested in a port must be referential",
                );
            }
            if flag(b, e, "isEnd")
                && (is(b, owner, "InterfaceDefinition") || is(b, owner, "InterfaceUsage"))
                && !is(b, e, "PortUsage")
            {
                let rule = if is(b, owner, "InterfaceDefinition") {
                    "validateInterfaceDefinitionEnd"
                } else {
                    "validateInterfaceUsageEnd"
                };
                report(e, span, rule, "An interface end must be a port");
            }
        }
        let required = match ty {
            "PerformActionUsage" => Some(("ActionUsage", "validatePerformActionUsageReference")),
            "ExhibitStateUsage" => Some(("StateUsage", "validateExhibitStateUsageReference")),
            "IncludeUseCaseUsage" => Some(("UseCaseUsage", "validateIncludeUseCaseUsageReference")),
            "SatisfyRequirementUsage" => Some((
                "RequirementUsage",
                "validateSatisfyRequirementUsageReference",
            )),
            "AssertConstraintUsage" => {
                Some(("ConstraintUsage", "validateAssertConstraintUsageReference"))
            }
            "EventOccurrenceUsage" => {
                Some(("OccurrenceUsage", "validateEventOccurrenceUsageReferent"))
            }
            _ => None,
        };
        if let Some((required, rule)) = required {
            for target in g.relations(b, e, "ReferenceSubsetting", "referencedFeature") {
                if !g.effective_kind(b, target, required) {
                    report(e, span, rule, &format!("{ty} must reference a {required}"));
                }
            }
        }
        if is(b, e, "Function") || is(b, e, "Expression") {
            let results = g
                .closure(e)
                .into_iter()
                .filter(|t| result_owners.contains(t))
                .count();
            if results > 1 {
                report(
                    e,
                    span,
                    if is(b, e, "Function") {
                        "validateFunctionResultExpressionMembership"
                    } else {
                        "validateExpressionResultExpressionMembership"
                    },
                    "Only one owned or inherited result expression is allowed",
                );
            }
            let returns = g.members[e]
                .iter()
                .filter(|&&m| {
                    b.elements[m]
                        .owning_relationship
                        .is_some_and(|r| b.elements[r].ty == "ReturnParameterMembership")
                })
                .count();
            if returns > 1 {
                report(
                    e,
                    span,
                    if is(b, e, "Function") {
                        "validateFunctionResultParameterMembership"
                    } else {
                        "validateExpressionResultParameterMembership"
                    },
                    "Only one return parameter is allowed",
                );
            }
        }
        if is(b, e, "Type")
            && g.members[e]
                .iter()
                .filter(|&&m| is(b, m, "Multiplicity"))
                .count()
                > 1
        {
            report(
                e,
                span,
                "validateTypeOwnedMultiplicity",
                "Only one multiplicity is allowed",
            );
        }
        for (rel, rule) in [
            (
                "ReferenceSubsetting",
                "validateFeatureOwnedReferenceSubsetting",
            ),
            ("CrossSubsetting", "validateFeatureOwnedCrossSubsetting"),
        ] {
            if b.elements[e]
                .owned_relationships
                .iter()
                .filter(|&&r| b.elements[r].ty == rel)
                .count()
                > 1
            {
                report(
                    e,
                    span,
                    rule,
                    "At most one relationship of this kind is allowed",
                );
            }
        }
        let chain = g.relations(b, e, "FeatureChaining", "chainingFeature");
        if b.elements[e]
            .owned_relationships
            .iter()
            .filter(|&&r| b.elements[r].ty == "FeatureChaining")
            .count()
            == 1
        {
            report(
                e,
                span,
                "validateFeatureChainingFeatureNotOne",
                "A feature chain must have at least two features",
            );
        }
        if chain.contains(&e) {
            report(
                e,
                span,
                "validateFeatureChainingFeaturesNotSelf",
                "A feature cannot chain through itself",
            );
        }
        if is(b, e, "StateDefinition") || is(b, e, "StateUsage") {
            let definition = is(b, e, "StateDefinition");
            for kind in ["entry", "do", "exit"] {
                let n = g.members[e]
                    .iter()
                    .filter(|&&m| {
                        b.elements[m].owning_relationship.is_some_and(|r| {
                            b.elements[r].ty == "StateSubactionMembership"
                                && b.elements[r].props.get("kind").and_then(|v| v.as_str())
                                    == Some(kind)
                        })
                    })
                    .count();
                if n > 1 {
                    report(
                        e,
                        span,
                        if definition {
                            "validateStateDefinitionSubactionKind"
                        } else {
                            "validateStateUsageSubactionKind"
                        },
                        "A state may have at most one subaction of each kind",
                    );
                }
            }
        }
        if is(b, e, "ControlNode")
            && !g.owner[e].is_some_and(|o| is(b, o, "ActionDefinition") || is(b, o, "ActionUsage"))
        {
            report(
                e,
                span,
                "validateControlNodeOwningType",
                "A control node must be owned by an action",
            );
        }
        if is(b, e, "Association") {
            let ends: Vec<_> = g.members[e]
                .iter()
                .copied()
                .filter(|&m| flag(b, m, "isEnd"))
                .collect();
            if ends.len() == 1 && g.supers[e].is_empty() {
                report(
                    e,
                    span,
                    "validateAssociationRelatedTypes",
                    "An association must have at least two related types",
                );
            }
            for end in ends {
                let types = g.typed(b, end);
                let minimal = types
                    .iter()
                    .filter(|&&t| !types.iter().any(|&u| u != t && g.closure(u).contains(&t)))
                    .count();
                if minimal > 1 {
                    report(
                        end,
                        g.span(b, end),
                        "validateAssociationEndTypes",
                        "An association end must have exactly one non-redundant type",
                    );
                }
            }
        }
        if b.elements[e]
            .owning_relationship
            .is_some_and(|r| b.elements[r].ty == "RequirementVerificationMembership")
        {
            let valid = g.owner[e].is_some_and(|o| {
                b.elements[o]
                    .owning_relationship
                    .is_some_and(|r| b.elements[r].ty == "ObjectiveMembership")
                    && g.owner[o].is_some_and(|c| {
                        is(b, c, "VerificationCaseDefinition") || is(b, c, "VerificationCaseUsage")
                    })
            });
            if !valid {
                report(
                    e,
                    span,
                    "validateRequirementVerificationMembershipOwningType",
                    "A requirement verification must be in the objective of a verification case",
                );
            }
        }
        if is(b, e, "MetadataFeature") {
            let types = g.typed(b, e);
            if !types.is_empty() && types.iter().all(|&t| flag(b, t, "isAbstract")) {
                report(
                    e,
                    span,
                    "validateMetadataFeatureMetadataNotAbstract",
                    "Metadata must have a concrete type",
                );
            }
        }
    }
    out
}

/// SysML selects ViewRenderingMembership from Type::featureMembership, which
/// includes inherited memberships after KerML's redefinition/visibility rules.
/// Use the shared projection rather than recounting all transitive declarations.
pub(super) fn validate_view_renderings(
    r: &mut ResolvedModel,
    model: &Model,
    g: &Facts,
) -> Vec<(usize, Diagnostic)> {
    let views: Vec<_> = (r.b.lib_boundary..r.b.explicit_len())
        .filter(|&e| {
            (is(&r.b, e, "ViewDefinition") || is(&r.b, e, "ViewUsage"))
                && !model.is_library_unit(r.b.unit_of_elem(e))
        })
        .collect();
    if views.is_empty() {
        return Vec::new();
    }
    let inputs = ViewRenderingInputs::new(r, model, g);
    let mut out = Vec::new();
    for e in views {
        let owned = r.b.elements[e]
            .owned_relationships
            .iter()
            .filter(|&&rel| is(&r.b, rel, "ViewRenderingMembership"))
            .count();
        // Owned memberships cannot be removed by inheritance/redefinition.
        // Retain this definite error even if another dependency is incomplete.
        let invalid = if owned > 1 {
            true
        } else {
            let inherited = r.inherited_memberships(ElementRef(e), true);
            owned
                + inherited
                    .iter()
                    .filter(|m| is(&r.b, m.0, "ViewRenderingMembership"))
                    .count()
                > 1
                && inputs.complete(r, g, e)
        };
        if invalid {
            out.push((
                r.b.unit_of_elem(e),
                super::rule_error(
                    g.span(&r.b, e),
                    if is(&r.b, e, "ViewDefinition") {
                        "validateViewDefinitionOnlyOnvViewRendering"
                    } else {
                        "validateViewUsageOnlyOneRendering"
                    },
                    "A view may have at most one rendering",
                ),
            ));
        }
    }
    out
}

/// Completeness preflight, not a second membership reducer. Missing written
/// bases and redefinitions are not all represented by inheritance_incomplete.
/// A later-resolved redefinition could remove a currently retained rendering,
/// so an incomplete candidate count cannot certify a violation.
struct ViewRenderingInputs {
    unresolved: std::collections::HashSet<usize>,
    recovered_units: std::collections::HashSet<usize>,
}
impl ViewRenderingInputs {
    fn new(r: &ResolvedModel, model: &Model, g: &Facts) -> Self {
        let mut unresolved = std::collections::HashSet::new();
        // A missing relationship endpoint affects its source, not every
        // enclosing namespace. Nested expression bodies do not change their
        // enclosing view's membership set. The dependency walk below follows
        // bases, import targets, owned features and redefinition chains.
        for reference in r.unresolved_references() {
            let e = reference.owner.0;
            unresolved.insert(e);
            if is(&r.b, e, "Relationship") {
                if let Some(owner) = g.owner.get(e).copied().flatten() {
                    unresolved.insert(owner);
                }
            }
        }
        let recovered_units = (0..model.unit_count())
            .filter(|&unit| {
                !model.is_library_unit(unit)
                    && model
                        .unit(unit)
                        .diagnostics
                        .iter()
                        .any(|d| d.severity == sysmlv2_syntax::diag::Severity::Error)
            })
            .collect();
        Self {
            unresolved,
            recovered_units,
        }
    }

    fn complete(&self, r: &mut ResolvedModel, g: &Facts, root: usize) -> bool {
        let mut steps = 0usize;
        let mut namespaces = vec![(root, false, false)];
        let mut seen_namespaces = std::collections::HashSet::new();
        let mut features = Vec::new();
        let mut seen_features = std::collections::HashSet::new();
        macro_rules! ready {
            ($e:expr) => {{
                let e = $e;
                steps = steps.saturating_add(1);
                if steps > crate::eval::MAX_STEPS
                    // Facts index the explicit graph. A generated element
                    // outside that snapshot has no certified dependencies here.
                    || e >= g.owner.len()
                    || self.unresolved.contains(&e)
                    || self.recovered_units.contains(&r.b.unit_of_elem(e))
                {
                    return false;
                }
            }};
        }
        while let Some((e, exports, recursive)) = namespaces.pop() {
            ready!(e);
            if !seen_namespaces.insert((e, exports, recursive)) {
                continue;
            }
            if r.inheritance_incomplete(ElementRef(e), true) {
                return false;
            }
            namespaces.extend(
                r.b.context_bases(e)
                    .into_iter()
                    .map(|base| (base, true, false)),
            );
            steps = steps.saturating_add(g.members[e].len());
            if steps > crate::eval::MAX_STEPS {
                return false;
            }
            for &member in &g.members[e] {
                if is(&r.b, member, "Feature") {
                    features.push(member);
                }
                if recursive && is(&r.b, member, "Namespace") {
                    namespaces.push((member, true, true));
                }
            }
            if exports {
                // Root imports are not Type::featureMembership. Bases and
                // namespace-import targets can re-export memberships. Audit
                // their dependencies even if a candidate was later suppressed.
                // Include private dependencies in this conservative audit:
                // an import-all path can admit them. The projection itself
                // decides visibility; this walk never adds memberships.
                for (target, namespace, recursive, _) in r.import_details(ElementRef(e)) {
                    if namespace {
                        namespaces.push((target.0, true, recursive));
                    } else {
                        features.push(target.0);
                        if recursive && is(&r.b, target.0, "Namespace") {
                            namespaces.push((target.0, true, true));
                        }
                    }
                }
            }
        }
        // Redefinition is transitive. An unavailable target on a removed
        // ancestor still affects the surviving feature's replacement set.
        while let Some(e) = features.pop() {
            ready!(e);
            if seen_features.insert(e) {
                features.extend(g.relations(&r.b, e, "Redefinition", "redefinedFeature"));
            }
        }
        true
    }
}

/// Direction conformance shares the same contextual proof as repair proposals.
pub(super) fn validate_directions(
    r: &mut ResolvedModel,
    model: &Model,
    g: &Facts,
) -> Vec<(usize, Diagnostic)> {
    let candidates: Vec<_> =
        r.b.spec_targets
            .iter()
            .enumerate()
            .filter_map(|(i, (e, kind, _, qn))| {
                if *kind != "Redefinition" || *e < r.b.lib_boundary {
                    return None;
                }
                let t = r.b.spec_resolved.get(i).copied().flatten()?;
                let unit = r.b.unit_of_elem(*e);
                (!model.is_library_unit(unit)).then_some((*e, t, unit, qn.span))
            })
            .collect();
    let mut proof = super::directions::RedefinitionDirections::default();
    candidates
        .into_iter()
        .filter(|&(e, t, _, _)| proof.conformance(&mut r.b, g, e, t) == Some(false))
        .map(|(_, _, unit, span)| {
            (
                unit,
                super::rule_error(
                    span,
                    "validateRedefinitionDirectionConformance",
                    "A redefining feature must have a compatible direction",
                ),
            )
        })
        .collect()
}

#[cfg(test)]
mod view_rendering_tests {
    use super::*;

    fn findings(source: &str) -> Vec<(String, String)> {
        let mut model = Model::new();
        model.add_source("views.sysml", source);
        assert!(
            model.unit(0).diagnostics.is_empty(),
            "{:?}",
            model.unit(0).diagnostics
        );
        let mut graph = ResolvedModel::build(&model);
        let facts = Facts::new(&mut graph.b);
        let first = validate_view_renderings(&mut graph, &model, &facts);
        let second = validate_view_renderings(&mut graph, &model, &facts);
        assert_eq!(first, second, "cached queries must preserve diagnostics");
        first
            .into_iter()
            .map(|(_, d)| {
                (
                    source[d.span.start as usize..d.span.end as usize].into(),
                    d.code.unwrap().into(),
                )
            })
            .collect()
    }

    #[test]
    fn inherited_renderings_include_same_names_and_imported_memberships() {
        let findings = findings(
            "package P {
            rendering def R;
            view def A { render rendering displayed : R; }
            view def B { protected render rendering another : R; }
            view def Same :> A { render rendering displayed : R; }
            view same : A { render rendering displayed : R; }
            view def Imported { public import A::displayed; }
            view def Combined :> Imported, B;
            view combined : Combined;
            view def LocalImport { public import A::displayed; render rendering local : R; }
        }",
        );
        assert_eq!(
            findings,
            [
                (
                    "Same".into(),
                    "validateViewDefinitionOnlyOneViewRendering".into()
                ),
                (
                    "same".into(),
                    "validateViewUsageOnlyOneViewRendering".into()
                ),
                (
                    "Combined".into(),
                    "validateViewDefinitionOnlyOneViewRendering".into()
                ),
                (
                    "combined".into(),
                    "validateViewUsageOnlyOneViewRendering".into()
                ),
            ]
        );
    }

    #[test]
    fn actual_redefinition_visibility_and_membership_identity_are_preserved() {
        assert!(
            findings(
                "package P {
            rendering def R;
            view def A { render rendering displayed : R; }
            view def Renamed :> A { render rendering replacement :>> A::displayed; }
            view def Transitive :> Renamed { render rendering again :>> Renamed::replacement; }
            view def Left :> A; view def Right :> A; view def Diamond :> Left, Right;
            view diamond : Diamond;
            view def Hidden { private render rendering hidden : R; }
            view def Child :> Hidden, A;
            view def Imported { private import A::displayed; }
            view def PrivateChild :> Imported, A;
        }"
            )
            .is_empty()
        );
    }

    #[test]
    fn incomplete_dependencies_do_not_hide_owned_errors_or_complete_peers() {
        let findings = findings(
            "package P {
            rendering def R;
            view def A { render rendering a : R; }
            view def B { render rendering b : R; }
            view def MissingBase :> A, B, Missing;
            view def MissingChild :> MissingBase;
            view def MissingRedefinition :> A, B { render rendering replacement :>> Missing::a; }
            view def MissingRedefinitionChild :> MissingRedefinition;
            view def MissingImport { public import Missing::*; }
            view def MissingImportChild :> MissingImport, A, B;
            view def CycleA :> CycleB, A; view def CycleB :> CycleA, B;
            view def Owned :> Missing { render rendering a : R; render rendering b : R; }
            view def CompletePeer :> A, B;
            view def ValidPeer :> A;
        }",
        );
        assert_eq!(
            findings.iter().map(|f| f.0.as_str()).collect::<Vec<_>>(),
            ["Owned", "CompletePeer"]
        );
    }

    #[test]
    fn unrelated_nested_bodies_do_not_change_the_membership_dependency_set() {
        let findings = findings(
            "package P {
            rendering def R;
            view def A {
                render rendering a : R;
                requirement req { attribute value : Missing; }
            }
            view def B { render rendering b : R; }
            view def Combined :> A, B;
        }",
        );
        assert_eq!(
            findings.iter().map(|f| f.0.as_str()).collect::<Vec<_>>(),
            ["Combined"]
        );
    }

    #[test]
    fn elements_added_after_the_static_index_cannot_supply_complete_inputs() {
        let mut model = Model::new();
        model.add_source("views.sysml", "view def V;");
        let mut graph = ResolvedModel::build(&model);
        let facts = Facts::new(&mut graph.b);
        let added = graph.b.elements.len();
        graph.b.elements.push(graph.b.elements[0].clone());
        let inputs = ViewRenderingInputs::new(&graph, &model, &facts);
        assert!(!inputs.complete(&mut graph, &facts, added));
    }
}

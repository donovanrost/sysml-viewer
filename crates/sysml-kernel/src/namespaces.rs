use crate::{Diagnostic, Element, Membership, Resolution, Severity, Stage};
use std::collections::{BTreeMap, HashMap, HashSet};
use sysmlv2_model::check::metaclass_conforms;

/// The upstream distinguishability pass omits aliases. A plain Membership's
/// isDistinguishableFrom requires nonoverlapping long/short names when the
/// member metaclasses conform in either direction. Check that side of each
/// pair containing an alias. Specialized membership overrides cannot make a
/// failing plain-Membership comparison true. Checked import closure prunes
/// colliding imports separately; this pass diagnoses owned/inherited aliases.
pub(crate) fn alias_collisions(
    elements: &[Element],
    memberships: &[Membership],
) -> Vec<Diagnostic> {
    let elements_by_id: HashMap<_, _> = elements.iter().map(|e| (e.id.as_str(), e)).collect();
    let memberships_by_id: HashMap<_, _> = memberships.iter().map(|m| (m.id.as_str(), m)).collect();
    let mut findings = Vec::new();
    for owner in elements.iter().filter(|e| e.members_available) {
        let Some(source) = &owner.source else {
            continue;
        };
        let mut names: BTreeMap<&str, Vec<&Membership>> = BTreeMap::new();
        for id in owner
            .owned_memberships
            .iter()
            .chain(&owner.inherited_memberships)
        {
            let member = memberships_by_id[id.as_str()];
            for name in member.name.iter().chain(&member.short_name) {
                let bucket = names.entry(name).or_default();
                if !bucket.iter().any(|m| m.id == member.id) {
                    bucket.push(member);
                }
            }
        }
        let mut reported = HashSet::new();
        for (name, matches) in names {
            for (index, a) in matches.iter().enumerate() {
                for b in &matches[index + 1..] {
                    if !a.is_alias && !b.is_alias {
                        continue;
                    }
                    let (
                        Resolution::Resolved { target_id: first },
                        Resolution::Resolved { target_id: second },
                    ) = (&a.member, &b.member)
                    else {
                        continue;
                    };
                    let first = &elements_by_id[first.as_str()].metaclass;
                    let second = &elements_by_id[second.as_str()].metaclass;
                    if !(metaclass_conforms(first, second) || metaclass_conforms(second, first)) {
                        continue;
                    }
                    let pair = if a.id < b.id {
                        (&a.id, &b.id)
                    } else {
                        (&b.id, &a.id)
                    };
                    if !reported.insert(pair) {
                        continue;
                    }
                    findings.push(Diagnostic {
                        code: "validateNamespaceDistinguishibility".into(),
                        stage: Stage::Semantic, severity: Severity::Error,
                        message: format!("Alias membership name `{name}` conflicts with another membership of this namespace whose member metaclass is not distinguishable."),
                        source: source.clone(),
                    });
                }
            }
        }
    }
    findings
}

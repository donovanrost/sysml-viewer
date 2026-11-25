use super::*;
use crate::{
    json::id_ref,
    model::{GraphFormat, Model},
    properties::Atom,
};

fn fixture(format: GraphFormat) -> ResolvedModel {
    let mut model = Model::with_graph_format(format);
    assert!(model.add_source("current-graph.kerml", "class B; class C; class A specializes B; feature f; feature g; function Parent {in a;return b;} function Child specializes Parent {in x;return y;}").diagnostics.is_empty());
    let mut r = ResolvedModel::build(&model);
    r.ensure_implied();
    r
}
fn clear_sources(row: &mut Elem) {
    for name in [
        "specific",
        "subclassifier",
        "typedFeature",
        "subsettingFeature",
        "redefiningFeature",
        "referencingFeature",
        "crossingFeature",
    ] {
        row.props
            .entries
            .make_mut()
            .retain(|(key, _)| key.name() != name);
    }
}

#[test]
fn streamed_current_graph_matches_full_reduction_for_aliases_carriers_tails_and_chains() {
    for format in [GraphFormat::LegacyV2, GraphFormat::CanonicalV3] {
        for variant in 0..12 {
            let mut r = fixture(format);
            let a = r.resolve_qualified("A").unwrap().0;
            let b = r.resolve_qualified("B").unwrap().0;
            let c = r.resolve_qualified("C").unwrap().0;
            let f = r.resolve_qualified("f").unwrap().0;
            let g = r.resolve_qualified("g").unwrap().0;
            let rel = *r.b.elements[a]
                .owned_relationships
                .iter()
                .find(|&&at| r.b.elements[at].ty == "Subclassification")
                .unwrap();
            let first_implied = r.b.implied.as_ref().unwrap().from;
            assert!(first_implied < r.b.elements.len());
            let before =
                super::super::structural_index::StoredStructure::get(&mut r.b, &mut 0).unwrap();
            match variant {
                0 => {}
                1 | 9 => {
                    let mut tail = r.b.elements[rel].clone();
                    tail.id = Uuid::new_v4();
                    tail.props
                        .insert("superclassifier", id_ref(r.b.elements[c].id));
                    r.b.elements.push(tail.clone());
                    tail.id = Uuid::new_v4();
                    tail.props
                        .insert("superclassifier", id_ref(r.b.elements[b].id));
                    r.b.elements.push(tail);
                    if variant == 9 {
                        // A true physical tail must survive recertification,
                        // even though no static-prefix recipe mentions it.
                        let mut tail = r.b.elements[rel].clone();
                        tail.id = Uuid::new_v4();
                        tail.props
                            .insert("subclassifier", id_ref(r.b.elements[c].id));
                        tail.props
                            .insert("superclassifier", id_ref(r.b.elements[a].id));
                        r.b.elements.push(tail);
                    }
                }
                2 => clear_sources(&mut r.b.elements[rel]),
                3 => r.b.elements[rel].props.insert("specific", Atom::Null),
                4 => {
                    let id = r.b.elements[c].id;
                    r.b.elements[rel].props.insert("specific", id_ref(id));
                }
                5 => r.b.elements[rel].props.insert("general", Atom::Null),
                6 => clear_sources(&mut r.b.elements[first_implied]),
                7 => r.b.elements[rel]
                    .props
                    .insert("specific", id_ref(Uuid::new_v4())),
                8 => {
                    let old = r.b.elements[b].id;
                    r.override_ids(&HashMap::from([(old, Uuid::new_v4())]));
                }
                10 => {
                    let target = r.b.elements[b].id;
                    let source = r.b.elements[g].id;
                    r.b.elements[f].ty = "Subclassification";
                    r.b.elements[f]
                        .props
                        .insert("subclassifier", id_ref(source));
                    r.b.elements[f]
                        .props
                        .insert("superclassifier", id_ref(target));
                }
                11 => r.b.elements[rel].ty = "Dependency",
                _ => unreachable!(),
            };
            assert_eq!(before.is_current(&r.b), variant == 0);
            let chains = super::super::type_relations::FeatureChainBases {
                targets: HashMap::from([(a, c), (f, g)]),
                ..Default::default()
            };
            let owners = r.b.semantic_relationship_owners();
            let expected =
                r.b.required_specialization_edges_with_chain_bases(
                    &[],
                    0,
                    r.b.elements.len(),
                    &owners,
                    &chains,
                    Some(&mut 0),
                )
                .unwrap()
                .1;
            let ids: Vec<_> = r.b.elements.iter().map(|row| row.id).collect();
            let actual = r.b.current_specialization_graph(&chains, &mut 0).unwrap();
            assert_eq!(actual, expected, "{format:?} variant {variant}");
            let indexed =
                super::super::structural_index::StoredStructure::get(&mut r.b, &mut 0).unwrap();
            assert_eq!(
                indexed.specialization_rows,
                r.b.elements
                    .iter()
                    .enumerate()
                    .filter_map(|(i, row)| conforms(row.ty, "Specialization").then_some(i))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                r.b.current_specialization_graph(&chains, &mut 0).unwrap(),
                expected
            );
            for end in [0, first_implied, r.b.elements.len()] {
                let indexed_graph =
                    r.b.required_specialization_edges_with_chain_bases(
                        &[],
                        0,
                        end,
                        &owners,
                        &chains,
                        Some(&mut 0),
                    )
                    .unwrap();
                r.b.stored_structure = None;
                let scanned_graph =
                    r.b.required_specialization_edges_with_chain_bases(
                        &[],
                        0,
                        end,
                        &owners,
                        &chains,
                        Some(&mut 0),
                    )
                    .unwrap();
                assert_eq!(
                    indexed_graph, scanned_graph,
                    "prefix {end}, {format:?}, variant {variant}"
                );
                r.b.stored_structure = Some(indexed.clone());
            }
            let mut complete_work = 0;
            assert_eq!(
                r.b.current_specialization_graph(&chains, &mut complete_work)
                    .unwrap(),
                expected
            );
            assert!(complete_work > 1);
            // The indexed path can legitimately finish with the old 20-step
            // allowance. Test exhaustion at the actual boundary instead.
            for remaining in [0, complete_work / 2, complete_work - 1] {
                let mut exhausted = crate::eval::MAX_STEPS - remaining;
                assert!(
                    r.b.current_specialization_graph(&chains, &mut exhausted)
                        .is_none()
                );
                assert_eq!(
                    r.b.current_specialization_graph(&chains, &mut 0).unwrap(),
                    expected
                );
            }
            assert_eq!(
                r.b.elements.iter().map(|row| row.id).collect::<Vec<_>>(),
                ids
            );
        }
    }
}

#[test]
fn deferred_variant_owners_and_skipped_requirement_merges_preserve_recipes_and_graph_order() {
    for format in [GraphFormat::LegacyV2, GraphFormat::CanonicalV3] {
        let mut model = Model::with_graph_format(format);
        assert!(
            model
                .add_library_source(
                    "base.kerml",
                    "standard library package Base {classifier Anything; feature things:Anything;}"
                )
                .diagnostics
                .is_empty()
        );
        assert!(
            model
                .add_library_source(
                    "parts.sysml",
                    "standard library package Parts {part def Part; part parts:Part;}"
                )
                .diagnostics
                .is_empty()
        );
        assert!(model.add_source("variants.sysml", "variation part def Choice {variant part option;} variation part choice {variant part option;} part def Plain;").diagnostics.is_empty());
        let mut r = ResolvedModel::build(&model);
        let names: HashMap<_, _> =
            r.b.lib_qnames
                .iter()
                .map(|(id, parts)| (parts.join("::"), *id))
                .collect();
        let required = r.b.supported_implied_specializations(&names, true);
        let owners = r.b.specialization_relation_owners();
        let mut expected_rows = Vec::new();
        let mut variant_rows = 0;
        for (&(owner, edge), keep) in required.candidates.iter() {
            if !keep {
                continue;
            }
            let slot = if owner >= r.b.lib_boundary {
                r.b.legacy_implied_plan_with_owners(owner, &names, &owners)
                    .iter()
                    .position(|old| *old == edge)
            } else {
                None
            };
            if r.b.elements[owner]
                .owning_relationship
                .is_some_and(|rel| r.b.elements[rel].ty == "VariantMembership")
            {
                assert!(slot.is_some());
                variant_rows += 1;
            }
            expected_rows.push(static_recipe(r.b.elements[owner].id, owner, edge, slot));
        }
        assert!(variant_rows >= 2);
        let prefix =
            r.b.static_specialization_prefix(&names, true, Some(&mut 0))
                .unwrap();
        assert_eq!(&prefix.rows[..expected_rows.len()], expected_rows);
        // Reference construction repeats every recipe, as the previous factory
        // did. Skipping already-present requirements must preserve Vec order.
        let mut expected_graph = HashMap::new();
        for (&source, targets) in &required.graph {
            expected_graph.insert(source, targets.clone());
        }
        for edge in &prefix.rows {
            let targets = expected_graph.entry(edge.owner_id).or_insert_with(Vec::new);
            if !targets.contains(&edge.target) {
                targets.push(edge.target);
            }
        }
        assert_eq!(prefix.specializations, expected_graph);
    }
}

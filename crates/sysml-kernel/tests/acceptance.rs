use std::sync::OnceLock;
use sysml_kernel::*;

fn analyze(documents: Vec<SourceDocument>, revision: u32) -> Snapshot {
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(move || {
            Engine::bundled()
                .analyze(
                    AnalysisRequest {
                        revision,
                        documents,
                    },
                    || false,
                )
                .unwrap()
        })
        .unwrap()
        .join()
        .unwrap()
}
fn doc(id: &str, text: &str) -> SourceDocument {
    SourceDocument {
        id: id.into(),
        name: format!("{id}.sysml"),
        text: text.into(),
    }
}
fn fixture() -> &'static Snapshot {
    static SNAPSHOT: OnceLock<Snapshot> = OnceLock::new();
    SNAPSHOT.get_or_init(|| {
        analyze(
            vec![
                doc(
                    "definitions",
                    include_str!("../../../tests/fixtures/kernel/Definitions.sysml"),
                ),
                doc(
                    "mission",
                    include_str!("../../../tests/fixtures/kernel/Mission.sysml"),
                ),
                doc(
                    "errors",
                    include_str!("../../../tests/fixtures/kernel/Errors.sysml"),
                ),
                doc(
                    "unicode",
                    "// 🚢\r\npackage '海' { part def 'Éngine'; part x : 'Éngine'; }",
                ),
                doc(
                    "broken-end",
                    "package Broken { part a; part b; connect a.missing to b; }",
                ),
                doc(
                    "conjugation",
                    "package Conjugation { port def P; part d { port p : ~ //* note */ P; port q : ~Absent; } }",
                ),
            ],
            7,
        )
    })
}
fn element<'a>(s: &'a Snapshot, name: &str) -> &'a Element {
    let matches = s.find(name);
    assert_eq!(matches.len(), 1, "{name}");
    matches[0]
}

#[test]
fn imported_type_and_standard_library_resolve() {
    let s = fixture();
    for (owner, target) in [
        ("Mission::vehicle::a", "Components::SpecialDevice"),
        ("Components::Device::mass", "ScalarValues::Real"),
    ] {
        let e = element(s, owner);
        let reference = s
            .references
            .iter()
            .find(|r| r.owner_id.as_deref() == Some(&e.id) && r.kind == RelationshipKind::Typing)
            .unwrap();
        assert_eq!(
            reference.resolution,
            Resolution::Resolved {
                target_id: element(s, target).id.clone()
            }
        );
    }
}

#[test]
fn conjugated_typing_preserves_written_marker_and_reference_range() {
    let s = fixture();
    for (name, spelling) in [("p", "~ //* note */ P"), ("q", "~Absent")] {
        let e = element(s, &format!("Conjugation::d::{name}"));
        let r = s
            .references
            .iter()
            .find(|r| r.owner_id.as_deref() == Some(&e.id) && r.kind == RelationshipKind::Typing)
            .unwrap();
        assert_eq!(r.spelling, spelling);
        let text = &s
            .documents
            .iter()
            .find(|d| d.id == "conjugation")
            .unwrap()
            .text;
        assert_eq!(
            &text[r.source.start.byte_offset as usize..r.source.end.byte_offset as usize],
            spelling
        );
        if name == "p" {
            assert!(matches!(r.resolution, Resolution::Resolved { .. }));
        } else {
            assert_eq!(r.resolution, Resolution::Unresolved);
        }
    }
}

#[test]
fn redefinition_replaces_inherited_port_without_losing_other_members() {
    let s = fixture();
    let e = element(s, "Mission::vehicle::a");
    let names: Vec<_> = e
        .effective_members
        .iter()
        .filter_map(|id| s.element(id))
        .filter(|m| !m.library)
        .map(|m| m.qualified_name.as_deref())
        .collect();
    assert!(names.contains(&Some("Components::SpecialDevice::signal")));
    assert!(names.contains(&Some("Components::Device::mass")));
    assert!(names.contains(&Some("Components::SpecialDevice::serial")));
    assert!(!names.contains(&Some("Components::Device::signal")));
    assert!(e.inheritance_complete);
}

#[test]
fn endpoints_retain_usage_context_and_resolved_port() {
    let s = fixture();
    let c = s
        .connections
        .iter()
        .find(|c| {
            s.element(&c.element_id)
                .unwrap()
                .source
                .as_ref()
                .unwrap()
                .document_id
                == "mission"
        })
        .unwrap();
    let names: Vec<_> = c.ends[0]
        .chain
        .iter()
        .map(|id| s.element(id).unwrap().qualified_name.as_deref().unwrap())
        .collect();
    assert_eq!(
        names,
        ["Mission::vehicle::a", "Components::SpecialDevice::signal"]
    );
    assert!(c
        .ends
        .iter()
        .all(|e| matches!(e.resolution, Resolution::Resolved { .. })));
}

#[test]
fn missing_endpoint_never_becomes_a_successful_prefix() {
    let s = fixture();
    let c = s
        .connections
        .iter()
        .find(|c| {
            s.element(&c.element_id)
                .unwrap()
                .source
                .as_ref()
                .unwrap()
                .document_id
                == "broken-end"
        })
        .unwrap();
    assert!(!matches!(c.ends[0].resolution, Resolution::Resolved { .. }));
}

#[test]
fn ambiguity_and_absence_are_distinct() {
    let s = fixture();
    assert!(s
        .references
        .iter()
        .any(|r| r.spelling == "Duplicate" && r.resolution == Resolution::Ambiguous));
    assert!(s
        .references
        .iter()
        .any(|r| r.spelling == "Absent" && r.resolution == Resolution::Unresolved));
    assert!(s
        .diagnostics
        .iter()
        .any(|d| d.code == "ambiguous-reference"));
}

#[test]
fn unicode_source_navigation_uses_utf16_and_exact_byte_ranges() {
    let s = fixture();
    let e = element(s, "'海'::x");
    let range = e.source.as_ref().unwrap();
    let text = &s.documents.iter().find(|d| d.id == "unicode").unwrap().text;
    assert_eq!(
        &text[range.start.byte_offset as usize..range.end.byte_offset as usize],
        "part x : 'Éngine';"
    );
    assert_eq!(
        text[..range.start.byte_offset as usize]
            .encode_utf16()
            .count() as u32,
        range.start.utf16_offset
    );
    assert_eq!(range.start.line, 2);
}

#[test]
fn every_snapshot_handle_and_source_range_is_valid() {
    let s = fixture();
    s.queries()
        .expect("Snapshot must satisfy the public query contract");
    let ids: std::collections::HashSet<_> = s.elements.iter().map(|e| &e.id).collect();
    assert_eq!(ids.len(), s.elements.len());
    for e in &s.elements {
        for id in e
            .parent_id
            .iter()
            .chain(&e.owned_members)
            .chain(&e.effective_members)
        {
            assert!(ids.contains(id), "dangling {id}");
        }
        if let Some(range) = &e.source {
            let text = &s
                .documents
                .iter()
                .find(|d| d.id == range.document_id)
                .unwrap()
                .text;
            assert!(text
                .get(range.start.byte_offset as usize..range.end.byte_offset as usize)
                .is_some());
        }
    }
    for r in &s.references {
        if let Resolution::Resolved { target_id } = &r.resolution {
            assert!(ids.contains(target_id));
        }
    }
}

#[test]
fn invalid_identity_and_cancellation_are_rejected() {
    let engine = Engine::bundled();
    let request = AnalysisRequest {
        revision: 0,
        documents: vec![doc("same", ""), doc("same", "")],
    };
    assert!(matches!(
        engine.analyze(request.clone(), || false),
        Err(AnalysisError::InvalidInput(_))
    ));
    assert!(matches!(
        engine.analyze(request, || true),
        Err(AnalysisError::Cancelled)
    ));
}

#[test]
fn recovery_and_revision_lifetimes() {
    let s = analyze(
        vec![doc(
            "malformed",
            "package P { part def A; part a : A; part broken : ; }",
        )],
        8,
    );
    assert!(s
        .diagnostics
        .iter()
        .any(|d| matches!(d.stage, Stage::Syntax)));
    assert_eq!(s.find("P::a").len(), 1);
    assert!(s.element(&fixture().elements[0].id).is_none());
}

#[test]
fn source_index_matches_unicode_reference_at_every_boundary() {
    let text = "a🚢海\r\nbé\n".repeat(100);
    let index = SourceIndex::new("doc", &text);
    for (offset, _) in text.char_indices() {
        let p = index.position(offset as u32);
        assert_eq!(p.byte_offset, offset as u32);
        assert_eq!(p.utf16_offset, text[..offset].encode_utf16().count() as u32);
        assert_eq!(
            p.line,
            text[..offset].chars().filter(|c| *c == '\n').count() as u32 + 1
        );
    }
}

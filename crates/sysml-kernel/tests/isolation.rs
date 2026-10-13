use sysml_kernel::*;

fn analyze(texts: &[&str]) -> Snapshot {
    let documents = texts
        .iter()
        .enumerate()
        .map(|(i, text)| SourceDocument {
            id: format!("source-{i}"),
            name: format!("source-{i}.sysml"),
            text: (*text).into(),
        })
        .collect();
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(move || {
            Engine::bundled()
                .analyze(
                    AnalysisRequest {
                        revision: 47,
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

#[test]
fn unresolved_bases_leave_independent_values_exact_in_either_file_order() {
    let text = include_str!("../../../tests/conformance/fixtures/mult-020.sysml");
    let (independent, broken) = text.split_at(text.find("package BrokenDomain").unwrap());
    for documents in [
        vec![text],
        vec![independent, broken],
        vec![broken, independent],
    ] {
        let snapshot = analyze(&documents);
        let q = snapshot.queries().unwrap();
        assert!(snapshot
            .diagnostics
            .iter()
            .any(|d| d.code == "resolution" && d.message.contains("MissingBase")));
        assert!(!snapshot
            .diagnostics
            .iter()
            .any(|d| matches!(d.stage, Stage::Syntax | Stage::Structure)));
        for (name, lower, upper) in [
            ("exact", "5", "5"),
            ("chosen", "10", "10"),
            ("range", "5", "10"),
        ] {
            let e = q.find(&format!("IndependentBounds::{name}"))[0];
            assert_eq!(
                q.cardinality(&e.id).unwrap(),
                &Cardinality::Evaluated {
                    lower: lower.into(),
                    upper: Some(upper.into())
                },
                "{name}"
            );
        }
        for name in ["Broken", "Indirect", "dependent"] {
            let e = q.find(&format!("BrokenDomain::{name}"))[0];
            assert!(!e.inheritance_complete, "{name}");
            assert!(
                matches!(q.cardinality(&e.id).unwrap(), Cardinality::Unknown { .. }),
                "{name}"
            );
        }
    }
}

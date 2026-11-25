//! Small, invented reproductions of editor/viewer disagreements. Expected
//! results follow KerML 7.2.2 names, 7.4.9.3 feature-chain expressions, and
//! 8.2.2.4 numeric tokens, rather than the behavior of a particular LSP.
use sysml_kernel::*;

#[test]
fn typed_member_chains_preserve_identity_visibility_and_numeric_precision() {
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(|| {
            for (dialect, definition, feature) in [
                ("sysml", "item def", "attribute"),
                ("kerml", "class", "feature"),
            ] {
                let definitions = format!(
                    "package Definitions {{
                        private import ScalarValues::*;
                        {definition} Payload {{
                            {feature} octets : Integer[*];
                            private {feature} hiddenOctets : Integer[*];
                        }}
                        {definition} ExtendedPayload :> Payload;
                    }}"
                );
                let receiver = if dialect == "sysml" {
                    "ref item"
                } else {
                    "feature"
                };
                let mut usages = format!(
                    "// 🚢 cross-file receiver\r\npackage Uses {{
                        private import Definitions::*;
                        private import ScalarValues::*;
                        private import SequenceFunctions::*;
                        {feature} missingOctets : Integer[*];
                        {definition} Packet {{
                            {receiver} 'frame' : ExtendedPayload;
                            {feature} valid = size('frame'.octets);
                            {feature} missing = size('frame'.missingOctets);
                            {feature} hidden = size('frame'.hiddenOctets);
                        }}"
                );
                let exact = [
                    "2147483648",
                    "4294967295",
                    "4294967296",
                    "9007199254740993",
                    "18446744073709551616",
                ];
                for (i, number) in exact.iter().enumerate() {
                    usages.push_str(&format!("{feature} n{i} : Integer[{number}];"));
                }
                usages.push_str(&format!(
                    "{feature} beyondBudget : Integer[340282366920938463463374607431768211456]; }}"
                ));
                let documents = vec![
                    SourceDocument {
                        id: "definitions".into(),
                        name: format!("definitions.{dialect}"),
                        text: definitions,
                    },
                    SourceDocument {
                        id: "usages".into(),
                        name: format!("usages.{dialect}"),
                        text: usages,
                    },
                ];
                // Resolution must be independent of document insertion order.
                for documents in [documents.clone(), documents.into_iter().rev().collect()] {
                    let snapshot = Engine::bundled()
                        .analyze(
                            AnalysisRequest {
                                revision: 1,
                                documents,
                            },
                            || false,
                        )
                        .unwrap();
                    let snapshot: Snapshot =
                        serde_json::from_slice(&serde_json::to_vec(&snapshot).unwrap()).unwrap();
                    assert_eq!(
                        snapshot.diagnostics.len(),
                        2,
                        "{dialect}: {:?}",
                        snapshot.diagnostics
                    );
                    assert!(snapshot
                        .diagnostics
                        .iter()
                        .all(|d| matches!(d.stage, Stage::Resolution)));
                    let queries = snapshot.queries().unwrap();
                    for (spelling, target) in [
                        ("'frame'", Some("Uses::Packet::frame")),
                        ("octets", Some("Definitions::Payload::octets")),
                        ("missingOctets", None),
                        ("hiddenOctets", None),
                    ] {
                        let refs: Vec<_> = snapshot
                            .references
                            .iter()
                            .filter(|r| r.source.document_id == "usages" && r.spelling == spelling)
                            .collect();
                        assert!(!refs.is_empty(), "{dialect}: {spelling}");
                        for reference in refs {
                            assert_eq!(queries.source(&reference.source).unwrap(), spelling);
                            let expected = match target {
                                Some(name) => Resolution::Resolved {
                                    target_id: queries.find(name)[0].id.clone(),
                                },
                                None => Resolution::Unresolved,
                            };
                            assert_eq!(reference.resolution, expected, "{dialect}: {spelling}");
                        }
                    }
                    for (i, number) in exact.iter().enumerate() {
                        let element = queries.find(&format!("Uses::n{i}"))[0];
                        assert_eq!(
                            queries.cardinality(&element.id).unwrap(),
                            &Cardinality::Evaluated {
                                lower: (*number).into(),
                                upper: Some((*number).into())
                            }
                        );
                    }
                    let element = queries.find("Uses::beyondBudget")[0];
                    assert_eq!(
                        queries.cardinality(&element.id).unwrap(),
                        &Cardinality::Unknown {
                            reason: CardinalityIssue::UnsupportedBound
                        }
                    );
                }
                let invalid = format!("{receiver} frame;");
                let report = check_syntax(SourceDocument {
                    id: "invalid".into(),
                    name: format!("invalid.{dialect}"),
                    text: invalid,
                })
                .unwrap();
                // `frame` is a SysML keyword, but a legal KerML basic name.
                assert_eq!(report.diagnostics.is_empty(), dialect == "kerml");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

use sysml_kernel::*;
use sysmlv2_syntax::{
    ast::{Dialect, ExprKind},
    parser::{is_reserved, parse_expression},
};

fn vocabulary() -> serde_json::Value {
    serde_json::from_str(include_str!("data/reserved-words.json")).unwrap()
}

fn words(vocabulary: &serde_json::Value, key: &str) -> Vec<String> {
    vocabulary[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap().to_string())
        .collect()
}

fn syntax(dialect: &str, text: &str, valid: bool) {
    let report = check_syntax(SourceDocument {
        id: "keywords".into(),
        name: format!("keywords.{dialect}"),
        text: text.into(),
    })
    .unwrap();
    assert_eq!(report.document.text, text);
    assert_eq!(
        report.diagnostics.is_empty(),
        valid,
        "{dialect}: {text}: {:?}",
        report.diagnostics
    );
}

#[test]
fn specified_reserved_words_and_provisional_constructor_keyword_are_dialect_specific() {
    let vocabulary = vocabulary();
    let mut candidates = words(&vocabulary, "sysml");
    candidates.extend(words(&vocabulary, "kerml"));
    candidates.extend(words(&vocabulary, "provisional"));
    candidates.sort();
    candidates.dedup();
    for (key, dialect) in [("sysml", Dialect::Sysml), ("kerml", Dialect::Kerml)] {
        let mut expected = words(&vocabulary, key);
        assert!(expected.windows(2).all(|w| w[0] < w[1]));
        expected.extend(words(&vocabulary, "provisional"));
        for word in &candidates {
            assert_eq!(
                is_reserved(dialect, word),
                expected.contains(word),
                "{key}: {word}"
            );
            for near in [
                format!("{word}_"),
                format!("_{word}"),
                format!("{word}0"),
                word.to_uppercase(),
            ] {
                assert!(!is_reserved(dialect, &near), "{key}: {near}");
                syntax(key, &format!("package {near};"), true);
            }
        }
    }
}

#[test]
fn every_reserved_word_requires_quotes_in_identification_aliases_and_references() {
    let vocabulary = vocabulary();
    for (dialect, feature) in [("sysml", "attribute"), ("kerml", "feature")] {
        let mut reserved = words(&vocabulary, dialect);
        reserved.extend(words(&vocabulary, "provisional"));
        for word in reserved {
            for quoted in [false, true] {
                let name = if quoted {
                    format!("'{word}'")
                } else {
                    word.clone()
                };
                for text in [
                    format!("package {name};"),
                    format!("package <{name}> P;"),
                    format!("alias {name} for P;"),
                    format!("alias A for P::{name};"),
                    format!("{feature} value = P::{name};"),
                ] {
                    syntax(dialect, &text, quoted);
                }
            }
        }
    }
}

#[test]
fn words_reserved_only_in_the_other_dialect_remain_usable_as_names() {
    let vocabulary = vocabulary();
    for (dialect, other, feature) in [
        ("sysml", "kerml", "attribute"),
        ("kerml", "sysml", "feature"),
    ] {
        let local = words(&vocabulary, dialect);
        for word in words(&vocabulary, other)
            .iter()
            .filter(|w| !local.contains(w))
        {
            for text in [
                format!("package <{word}> P;"),
                format!("package {word};"),
                format!("alias {word} for P;"),
                format!("alias A for P::{word};"),
                format!("{feature} value = P::{word};"),
            ] {
                syntax(dialect, &text, true);
            }
        }
    }
}

#[test]
fn reserved_short_name_diagnostics_locate_the_token_after_unicode_trivia() {
    let vocabulary = vocabulary();
    for dialect in ["sysml", "kerml"] {
        for word in words(&vocabulary, dialect) {
            let prefix = "// 海🚢\r\npackage <";
            let text = format!("{prefix}{word}> P; package Recovered;");
            let report = check_syntax(SourceDocument {
                id: "keywords".into(),
                name: format!("keywords.{dialect}"),
                text,
            })
            .unwrap();
            let diagnostic = report
                .diagnostics
                .iter()
                .find(|d| matches!(d.stage, Stage::Syntax) && d.message.contains("expected a name"))
                .expect("reserved short name must produce a syntax diagnostic");
            assert_eq!(diagnostic.severity, Severity::Error);
            let range = &diagnostic.source;
            assert_eq!(range.start.byte_offset as usize, prefix.len());
            assert_eq!(range.end.byte_offset as usize, prefix.len() + word.len());
            assert_eq!(
                range.start.utf16_offset as usize,
                prefix.encode_utf16().count()
            );
            assert_eq!(
                range.end.utf16_offset as usize,
                prefix.encode_utf16().count() + word.len()
            );
            assert_eq!(range.start.line, 2);
            assert_eq!(range.start.column, 10);
        }
    }
}

#[test]
fn constructor_keyword_policy_preserves_constructors_and_quoted_name_expressions() {
    // Published reserved-word lists omit `new`, but the constructor production
    // uses it. Keep this compatibility policy separate from normative coverage
    // pending OMG SYSML21-641 and KERML11-200.
    for text in ["new T()", "new P::T(1)", "new 'new'()", "new //* 🚢 */ T()"] {
        let parsed = parse_expression(text);
        assert!(
            parsed.diagnostics.is_empty(),
            "{text}: {:?}",
            parsed.diagnostics
        );
        let expression = parsed.expr.unwrap();
        assert!(matches!(expression.kind, ExprKind::Constructor { .. }));
        assert_eq!(expression.span.slice(text), text);
        for (dialect, feature) in [("sysml", "attribute"), ("kerml", "feature")] {
            syntax(dialect, &format!("{feature} value = {text};"), true);
        }
    }
    let reference = parse_expression("'new'").expr.unwrap();
    assert!(matches!(reference.kind, ExprKind::Ref(_)));
    let invocation = parse_expression("'new'()").expr.unwrap();
    assert!(matches!(invocation.kind, ExprKind::Invocation { .. }));
    for text in ["new", "new()", "new T("] {
        let parsed = parse_expression(text);
        assert!(parsed.expr.is_none(), "{text}");
        assert!(!parsed.diagnostics.is_empty(), "{text}");
    }
}

#[test]
fn quoted_keywords_and_dialect_names_survive_snapshot_queries_and_roundtrip() {
    fn check() {
        let documents = [
            (
                "lex-010.sysml",
                include_str!("../../../tests/conformance/fixtures/lex-010.sysml"),
            ),
            (
                "lex-011.kerml",
                include_str!("../../../tests/conformance/fixtures/lex-011.kerml"),
            ),
        ]
        .into_iter()
        .map(|(name, text)| SourceDocument {
            id: name.into(),
            name: name.into(),
            text: text.into(),
        })
        .collect();
        let snapshot = Engine::bundled()
            .analyze(
                AnalysisRequest {
                    revision: 40,
                    documents,
                },
                || false,
            )
            .unwrap();
        assert!(
            snapshot.diagnostics.is_empty(),
            "{:?}",
            snapshot.diagnostics
        );
        let encoded = serde_json::to_vec(&snapshot).unwrap();
        let decoded: Snapshot = serde_json::from_slice(&encoded).unwrap();
        let query = decoded.queries().unwrap();
        for package in ["KeywordsSysML", "KeywordsKerML"] {
            let element = query.find(&format!("{package}::bound"))[0];
            assert_eq!(
                query.cardinality(&element.id).unwrap(),
                &Cardinality::Evaluated {
                    lower: "3".into(),
                    upper: Some("3".into()),
                }
            );
            for name in [
                "quotedType",
                "viaShort",
                "viaAlias",
                "viaAliasShort",
                "foreign",
                "uppercase",
                "quotedNew",
            ] {
                let element = query.find(&format!("{package}::{name}"))[0];
                let references: Vec<_> = decoded
                    .references
                    .iter()
                    .filter(|r| r.owner_id.as_deref() == Some(element.id.as_str()))
                    .collect();
                assert_eq!(references.len(), 1, "{package}::{name}");
                assert!(matches!(
                    references[0].resolution,
                    Resolution::Resolved { .. }
                ));
            }
        }
    }
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(check)
        .unwrap()
        .join()
        .unwrap();
}

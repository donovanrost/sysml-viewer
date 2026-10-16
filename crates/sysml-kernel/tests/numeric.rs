use sysmlv2_model::{json::to_compact_json, rational::Rational};
use sysmlv2_syntax::{
    ast::{BinaryOp, ExprKind, Literal, UnaryOp},
    lexer::tokenize,
    parser::{parse_expression, parse_kerml_source, parse_source, Parse},
    token::TokenKind,
};

#[test]
fn real_values_exclude_trivia_without_losing_source_spans() {
    for separator in [
        "",
        " ",
        "\t",
        "\u{000c}",
        "\r\n",
        "//* 海🚢 */",
        "// 海🚢\n",
    ] {
        for (whole, fraction) in [
            ("1", "25"),
            ("", "5"),
            ("0001", "2500e+02"),
            ("", "5E-2"),
            ("9007199254740993", "00000000000000000001"),
        ] {
            let literal = format!("{whole}{separator}.{separator}{fraction}");
            let source = format!("// before 🚢\n{literal} // after");
            let parsed = parse_expression(&source);
            assert!(
                parsed.diagnostics.is_empty(),
                "{source}: {:?}",
                parsed.diagnostics
            );
            let expression = parsed.expr.unwrap();
            let expected = format!("{whole}.{fraction}");
            assert_eq!(
                expression.kind,
                ExprKind::Literal(Literal::Real(expected.clone()))
            );
            // With no whole-number part, the literal starts at the decimal point.
            assert_eq!(
                expression.span.slice(&source),
                literal.trim_start_matches(separator)
            );
            let ExprKind::Literal(Literal::Real(value)) = expression.kind else {
                unreachable!()
            };
            assert_eq!(
                Rational::parse_decimal(&value),
                Rational::parse_decimal(&expected)
            );
            assert!(Rational::parse_decimal(&value).is_some());
        }
    }
}

#[test]
fn extreme_decimal_scales_do_not_wrap_or_panic() {
    for text in [
        "1e-9223372036854775808",
        "0e-9223372036854775808",
        "1.0e-9223372036854775808",
        "1e9223372036854775807",
        "1e9223372036854775808",
        "1e-9223372036854775809",
        "1e100001",
        "1e-100001",
        "0.1e-100000",
    ] {
        assert!(Rational::parse_decimal(text).is_none(), "{text}");
    }
    for text in ["1e100000", "1e-100000", "1.0e100001", "0.1e-99999"] {
        assert!(Rational::parse_decimal(text).is_some(), "{text}");
    }
}

#[test]
fn numeric_tokens_keep_exponents_contiguous_and_ranges_distinct() {
    use TokenKind::*;
    for (source, expected) in [
        ("000123", vec![Decimal]),
        ("1e2 1E+2 1e-2", vec![Exp, Exp, Exp]),
        ("1..5", vec![Decimal, DotDot, Decimal]),
        (
            "1.25..2.5",
            vec![Decimal, Dot, Decimal, DotDot, Decimal, Dot, Decimal],
        ),
        ("-1.25e-2", vec![Minus, Decimal, Dot, Exp]),
        ("1e+", vec![Decimal, Ident, Plus]),
        ("1 e2", vec![Decimal, Ident]),
        ("1e + 2", vec![Decimal, Ident, Plus, Decimal]),
    ] {
        let (tokens, diagnostics) = tokenize(source);
        assert!(diagnostics.is_empty());
        let actual: Vec<_> = tokens
            .iter()
            .filter(|t| !t.kind.is_trivia() && t.kind != Eof)
            .map(|t| t.kind)
            .collect();
        assert_eq!(actual, expected, "{source}");
        let mut end = 0;
        for token in tokens {
            assert_eq!(token.span.start, end);
            end = token.span.end;
            assert!(source.is_char_boundary(end as usize));
        }
        assert_eq!(end as usize, source.len());
    }
}

#[test]
fn signs_and_ranges_are_expressions_and_invalid_numeric_forms_are_rejected() {
    for (source, op) in [("-1.25", UnaryOp::Minus), ("+ .5e2", UnaryOp::Plus)] {
        let parsed = parse_expression(source);
        assert!(parsed.diagnostics.is_empty());
        assert!(
            matches!(parsed.expr.unwrap().kind, ExprKind::Unary { op: actual, .. } if actual == op)
        );
    }
    for source in ["1..5", "1 . 25 .. 2 . 5"] {
        let parsed = parse_expression(source);
        assert!(parsed.diagnostics.is_empty());
        assert!(matches!(
            parsed.expr.unwrap().kind,
            ExprKind::Binary {
                op: BinaryOp::Range,
                ..
            }
        ));
    }
    for source in [
        "1.",
        ".",
        ".e2",
        "1e",
        "1e+",
        "1.0e-",
        "1 e2",
        "1e + 2",
        "1_000",
        "0x10",
        "1 /* regular comment */ . 25",
    ] {
        let parsed = parse_expression(source);
        assert!(parsed.expr.is_none(), "{source}");
        assert!(!parsed.diagnostics.is_empty(), "{source}");
        for (name, declaration) in [("invalid.sysml", "attribute"), ("invalid.kerml", "feature")] {
            let text = format!("// 海🚢\r\n{declaration} x = {source}; {declaration} recovered;");
            let report = sysml_kernel::check_syntax(sysml_kernel::SourceDocument {
                id: name.into(),
                name: name.into(),
                text,
            })
            .unwrap();
            assert!(!report.diagnostics.is_empty(), "{name}: {source}");
            for d in report.diagnostics {
                let range = &d.source;
                let text = &report.document.text;
                let start = range.start.byte_offset as usize;
                let end = range.end.byte_offset as usize;
                assert!(text.is_char_boundary(start) && text.is_char_boundary(end));
                assert_eq!(
                    text[..start].encode_utf16().count(),
                    range.start.utf16_offset as usize
                );
                assert_eq!(
                    text[..end].encode_utf16().count(),
                    range.end.utf16_offset as usize
                );
            }
        }
    }
}

#[test]
fn lowering_preserves_exact_numeric_values_in_both_dialects() {
    for (declaration, parser) in [
        ("attribute", parse_source as fn(&str) -> Parse),
        ("feature", parse_kerml_source as fn(&str) -> Parse),
    ] {
        for (literal, expected) in [
            ("1 //* 海🚢 */ . // fraction\n 25", "1.25"),
            (". \u{000c} 5E+2", ".5E+2"),
            (
                "9007199254740993 . 00000000000000000001",
                "9007199254740993.00000000000000000001",
            ),
            ("1e-9223372036854775808", "1e-9223372036854775808"),
        ] {
            let parsed = parser(&format!("{declaration} x = {literal};"));
            assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
            let json = to_compact_json(&parsed.unit);
            let rows: Vec<_> = json
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["@type"] == "LiteralRational")
                .collect();
            assert_eq!(rows.len(), 1);
            let value = &rows[0]["value"];
            let encoded = value
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| value.to_string());
            if let Some(exact) = Rational::parse_decimal(expected) {
                assert_eq!(Rational::parse_decimal(&encoded), Some(exact), "{literal}");
            } else {
                assert_eq!(encoded, expected);
            }
            if expected.len() > 25 {
                assert!(value.is_string(), "{literal}");
            }
        }
    }
}

#[test]
fn numeric_source_queries_and_recovery_survive_snapshot_round_trip() {
    use sysml_kernel::*;
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(|| {
            for (fixtures, invalid) in [
                (
                    vec![
                        (
                            "lex-006.sysml",
                            include_str!("../../../tests/conformance/fixtures/lex-006.sysml"),
                        ),
                        (
                            "lex-007.kerml",
                            include_str!("../../../tests/conformance/fixtures/lex-007.kerml"),
                        ),
                    ],
                    false,
                ),
                (
                    vec![
                        (
                            "lex-008.sysml",
                            include_str!("../../../tests/conformance/fixtures/lex-008.sysml"),
                        ),
                        (
                            "lex-009.kerml",
                            include_str!("../../../tests/conformance/fixtures/lex-009.kerml"),
                        ),
                        (
                            "lex-005.sysml",
                            include_str!("../../../tests/conformance/fixtures/lex-005.sysml"),
                        ),
                    ],
                    true,
                ),
            ] {
                let snapshot = Engine::bundled()
                    .analyze(
                        AnalysisRequest {
                            revision: 39,
                            documents: fixtures
                                .into_iter()
                                .map(|(name, text)| SourceDocument {
                                    id: name.into(),
                                    name: name.into(),
                                    text: text.into(),
                                })
                                .collect(),
                        },
                        || false,
                    )
                    .unwrap();
                let snapshot: Snapshot =
                    serde_json::from_slice(&serde_json::to_vec(&snapshot).unwrap()).unwrap();
                let query = snapshot.queries().unwrap();
                if invalid {
                    assert!(snapshot
                        .diagnostics
                        .iter()
                        .any(|d| matches!(d.stage, Stage::Syntax)));
                    for name in [
                        "InvalidNumericSysML::recovered",
                        "InvalidNumericKerML::recovered",
                        "LexicalDependent::clean",
                    ] {
                        let e = query.find(name)[0];
                        assert_eq!(
                            query.cardinality(&e.id).unwrap(),
                            &Cardinality::Unknown {
                                reason: CardinalityIssue::InvalidSyntax
                            }
                        );
                    }
                } else {
                    assert_eq!(snapshot.diagnostics.len(), 2, "{:?}", snapshot.diagnostics);
                    for diagnostic in &snapshot.diagnostics {
                        assert!(matches!(diagnostic.stage, Stage::Semantic));
                        assert!(diagnostic
                            .message
                            .contains("multiplicity bound must be a Natural number"));
                    }
                    for (package, declaration) in
                        [("NumericSysML", "attribute"), ("NumericKerML", "feature")]
                    {
                        let e = query.find(&format!("{package}::spaced"))[0];
                        assert_eq!(e.value_expression.as_deref(), Some("1 . 25"));
                        assert_eq!(
                            query.source(e.source.as_ref().unwrap()).unwrap(),
                            format!("{declaration} spaced = 1 . 25;")
                        );
                        let e = query.find(&format!("{package}::fixed"))[0];
                        assert_eq!(
                            query.cardinality(&e.id).unwrap(),
                            &Cardinality::Evaluated {
                                lower: "12".into(),
                                upper: Some("12".into())
                            }
                        );
                        for name in ["realBound", "extremeBound"] {
                            let e = query.find(&format!("{package}::{name}"))[0];
                            assert_eq!(
                                query.cardinality(&e.id).unwrap(),
                                &Cardinality::Unknown {
                                    reason: CardinalityIssue::UnsupportedBound
                                }
                            );
                        }
                    }
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

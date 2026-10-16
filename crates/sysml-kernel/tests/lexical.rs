use sysml_kernel::*;

fn document(name: &str, text: &str) -> SourceDocument {
    SourceDocument {
        id: name.into(),
        name: name.into(),
        text: text.into(),
    }
}

#[test]
fn whitespace_and_all_standard_escapes_parse_in_both_dialects() {
    for (dialect, declaration, feature) in [
        ("sysml", "part def", "attribute"),
        ("kerml", "class", "feature"),
    ] {
        for escape in ['b', 'f', 't', 'n', '\'', '"', '\\'] {
            let text = format!("package\u{000c}P {{ {declaration} '海🚢\\{escape}'; {feature} text = \"海🚢\\{escape}\"; }}");
            let report = check_syntax(document(&format!("escape.{dialect}"), &text)).unwrap();
            assert!(
                report.diagnostics.is_empty(),
                "{text}: {:?}",
                report.diagnostics
            );
        }
        let declaration = declaration.replace(' ', "\u{000c}");
        let text = format!("package \t\u{000c}\r\nP {{ {declaration}\u{000c}A; }}");
        assert!(
            check_syntax(document(&format!("whitespace.{dialect}"), &text))
                .unwrap()
                .diagnostics
                .is_empty()
        );
    }
}

#[test]
fn quoted_control_and_escape_diagnostics_keep_exact_byte_and_utf16_ranges() {
    let mut invalid: Vec<(String, &str)> = (0..=0x1f)
        .chain(0x7f..=0x9f)
        .map(|code| {
            (
                char::from_u32(code).unwrap().to_string(),
                "unescaped control character",
            )
        })
        .collect();
    invalid.extend(
        ['r', 'v', 'a', '0', 'u', 'x', '/', '海', '🚢']
            .map(|c| (format!("\\{c}"), "invalid escape sequence")),
    );
    for (dialect, declaration, feature) in [
        ("sysml", "part def", "attribute"),
        ("kerml", "class", "feature"),
    ] {
        for quote in ['\'', '"'] {
            for (invalid, message) in &invalid {
                let prefix = if quote == '\'' {
                    format!("// 🚢\r\n{declaration} '")
                } else {
                    format!("// 🚢\r\n{feature} value = \"")
                };
                let prefix = format!("{prefix}海🚢");
                let text = format!("{prefix}{invalid}end{quote}; {declaration} Recovered;");
                let report = check_syntax(document(&format!("invalid.{dialect}"), &text)).unwrap();
                let diagnostic = report
                    .diagnostics
                    .iter()
                    .find(|d| d.message.contains(message))
                    .unwrap();
                assert!(matches!(diagnostic.stage, Stage::Syntax));
                assert_eq!(diagnostic.severity, Severity::Error);
                assert_eq!(diagnostic.source.start.byte_offset as usize, prefix.len());
                assert_eq!(
                    diagnostic.source.end.byte_offset as usize,
                    prefix.len() + invalid.len()
                );
                assert_eq!(
                    diagnostic.source.start.utf16_offset as usize,
                    prefix.encode_utf16().count()
                );
                assert_eq!(
                    diagnostic.source.end.utf16_offset as usize,
                    prefix.encode_utf16().count() + invalid.encode_utf16().count()
                );
                assert_eq!(diagnostic.source.start.line, 2);
                assert_eq!(
                    diagnostic.source.start.column as usize,
                    prefix.rsplit('\n').next().unwrap().encode_utf16().count() + 1
                );
            }
        }
    }
}

#[test]
fn lexical_recovery_preserves_declarations_and_invalidates_workspace_bounds() {
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let snapshot = Engine::bundled()
                .analyze(
                    AnalysisRequest {
                        revision: 37,
                        documents: vec![
                            document(
                                "lex-003.sysml",
                                include_str!("../../../tests/conformance/fixtures/lex-003.sysml"),
                            ),
                            document(
                                "lex-004.kerml",
                                include_str!("../../../tests/conformance/fixtures/lex-004.kerml"),
                            ),
                            document(
                                "lex-005.sysml",
                                include_str!("../../../tests/conformance/fixtures/lex-005.sysml"),
                            ),
                        ],
                    },
                    || false,
                )
                .unwrap();
            let encoded = serde_json::to_vec(&snapshot).unwrap();
            let decoded: Snapshot = serde_json::from_slice(&encoded).unwrap();
            let query = decoded.queries().unwrap();
            for name in [
                "InvalidLexicalSysML::recovered",
                "InvalidLexicalKerML::recovered",
                "LexicalDependent::clean",
            ] {
                let element = query.find(name)[0];
                assert!(matches!(
                    query.cardinality(&element.id).unwrap(),
                    Cardinality::Unknown {
                        reason: CardinalityIssue::InvalidSyntax
                    }
                ));
            }
            for name in ["lex-003.sysml", "lex-004.kerml", "lex-005.sysml"] {
                let document = query.document(name).unwrap();
                for diagnostic in query.diagnostics(name).unwrap() {
                    let range = &diagnostic.source;
                    assert!(document
                        .text
                        .is_char_boundary(range.start.byte_offset as usize));
                    assert!(document
                        .text
                        .is_char_boundary(range.end.byte_offset as usize));
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

use sysmlv2_syntax::{lexer::tokenize, parser::parse_source};

#[test]
fn token_spans_preserve_every_byte_of_malformed_and_unicode_sources() {
    // Deterministic small-input property sweep. This supplements specific parser
    // acceptance cases; it is not a substitute for a continuous fuzz campaign.
    let alphabet = [
        'a', ' ', '\n', '\r', '\t', '\'', '"', '/', '*', '{', '}', ';', '\\', '海', '🚢',
    ];
    let mut seed = 0x5a17_u64;
    for length in 0..256 {
        let text: String = (0..length)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                alphabet[(seed as usize) % alphabet.len()]
            })
            .collect();
        let (tokens, _) = tokenize(&text);
        let mut offset = 0;
        for token in tokens {
            assert_eq!(token.span.start, offset);
            assert!(text
                .get(token.span.start as usize..token.span.end as usize)
                .is_some());
            offset = token.span.end;
        }
        assert_eq!(offset as usize, text.len());
        let parsed = parse_source(&text);
        for diagnostic in parsed.diagnostics {
            assert!(text
                .get(diagnostic.span.start as usize..diagnostic.span.end as usize)
                .is_some());
        }
    }
}

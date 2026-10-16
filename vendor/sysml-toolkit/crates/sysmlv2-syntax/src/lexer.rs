//! Tokenizer for the KerML / SysML v2 textual notation.
//!
//! Implements the shared terminal rules, with whitespace and quoted values
//! checked against KerML 1.0 8.2.2.1, 8.2.2.3 and 8.2.2.5:
//!
//! ```text
//! DECIMAL_VALUE     : '0'..'9' ('0'..'9')*
//! EXP_VALUE         : DECIMAL_VALUE ('e'|'E') ('+'|'-')? DECIMAL_VALUE
//! ID                : ('a'..'z'|'A'..'Z'|'_') ('a'..'z'|'A'..'Z'|'_'|'0'..'9')*
//! UNRESTRICTED_NAME : '\'' (ESCAPE | ~('\\'|'\''))* '\''
//! STRING_VALUE      : '"'  (ESCAPE | ~('\\'|'"'))* '"'
//! REGULAR_COMMENT   : '/*' -> '*/'
//! ML_NOTE           : '//*' -> '*/'
//! SL_NOTE           : '//' ...to end of line
//! WS                : (' '|'\t'|'\u{000C}'|'\r'|'\n')+
//! ESCAPE            : '\\' ('b'|'t'|'n'|'f'|'"'|'\''|'\\')
//! ```
//!
//! The quoted-token bodies additionally reject raw control characters (C0/C1).
//! This does not define an exhaustive Unicode printable-character repertoire.
//!
//! Note that a real literal like `1.5` is *not* a single token: per the
//! grammar it is `DECIMAL '.' DECIMAL`, composed by the parser (which is what
//! makes `1..5` lex cleanly as a range).

use crate::diag::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// The most token slots reserved before tokenizing, whatever the input
/// size (see the estimate below).
const MAX_TOKEN_HINT: usize = 64 * 1024;

/// Tokenize `src`, returning every token including trivia, terminated by an
/// [`TokenKind::Eof`] token. Malformed input yields `Error` tokens plus
/// diagnostics; the token stream always covers the entire input.
#[must_use]
pub fn tokenize(src: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut lexer = Lexer {
        src: src.as_bytes(),
        text: src,
        pos: 0,
        // Measured across the corpora, source text runs about two bytes
        // to the token once trivia is counted. The estimate is capped
        // because the ratio is a property of written notation, not of
        // text: a file that is mostly one long comment is a couple of
        // tokens, and a slot per two bytes of it would reserve orders of
        // magnitude more than the lexer will fill. Past the cap the
        // vector grows the usual way.
        tokens: Vec::with_capacity((src.len() / 2 + 1).min(MAX_TOKEN_HINT)),
        diags: Vec::new(),
    };
    lexer.run();
    (lexer.tokens, lexer.diags)
}

struct Lexer<'s> {
    src: &'s [u8],
    text: &'s str,
    pos: usize,
    tokens: Vec<Token>,
    diags: Vec<Diagnostic>,
}

impl<'s> Lexer<'s> {
    fn run(&mut self) {
        while self.pos < self.src.len() {
            let start = self.pos;
            let kind = self.next_kind();
            debug_assert!(self.pos > start, "lexer must always make progress");
            self.tokens.push(Token::new(kind, self.span_from(start)));
        }
        let end = crate::span::clamp_offset(self.src.len());
        self.tokens
            .push(Token::new(TokenKind::Eof, Span::new(end, end)));
    }

    fn span_from(&self, start: usize) -> Span {
        Span::new(
            crate::span::clamp_offset(start),
            crate::span::clamp_offset(self.pos),
        )
    }

    fn peek(&self, ahead: usize) -> Option<u8> {
        self.src.get(self.pos + ahead).copied()
    }

    fn bump(&mut self) -> u8 {
        let b = self.src[self.pos];
        self.pos += 1;
        b
    }

    fn eat(&mut self, b: u8) -> bool {
        if self.peek(0) == Some(b) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn error(&mut self, start: usize, message: impl Into<String>) {
        self.diags
            .push(Diagnostic::error(self.span_from(start), message));
    }

    fn next_kind(&mut self) -> TokenKind {
        use TokenKind::*;
        let start = self.pos;
        let b = self.bump();
        match b {
            b' ' | b'\t' | b'\x0c' | b'\r' | b'\n' => {
                while matches!(self.peek(0), Some(b' ' | b'\t' | b'\x0c' | b'\r' | b'\n')) {
                    self.pos += 1;
                }
                Whitespace
            }

            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while matches!(
                    self.peek(0),
                    Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
                ) {
                    self.pos += 1;
                }
                Ident
            }

            b'0'..=b'9' => self.number(),

            b'\'' => self.quoted(start, b'\'', "unrestricted name"),
            b'"' => self.quoted(start, b'"', "string literal"),

            b'/' => match self.peek(0) {
                // REGULAR_COMMENT: /* ... */
                Some(b'*') => {
                    self.pos += 1;
                    if self.eat_until_star_slash() {
                        RegularComment
                    } else {
                        self.error(start, "unterminated comment (missing `*/`)");
                        Error
                    }
                }
                Some(b'/') => {
                    self.pos += 1;
                    if self.peek(0) == Some(b'*') {
                        // ML_NOTE: //* ... */
                        self.pos += 1;
                        if self.eat_until_star_slash() {
                            BlockNote
                        } else {
                            self.error(start, "unterminated note (missing `*/`)");
                            Error
                        }
                    } else {
                        // SL_NOTE: // to end of line (newline stays outside)
                        while !matches!(self.peek(0), None | Some(b'\n' | b'\r')) {
                            self.pos += 1;
                        }
                        LineNote
                    }
                }
                _ => Slash,
            },

            b';' => Semi,
            b',' => Comma,
            b'.' => {
                if self.eat(b'.') {
                    DotDot
                } else if self.eat(b'?') {
                    DotQuestion
                } else {
                    Dot
                }
            }
            b'?' => {
                if self.eat(b'?') {
                    QuestionQuestion
                } else {
                    Question
                }
            }
            b'~' => Tilde,
            b'=' => {
                if self.eat(b'=') {
                    if self.eat(b'=') { EqEqEq } else { EqEq }
                } else if self.eat(b'>') {
                    FatArrow
                } else {
                    Eq
                }
            }
            b'!' => {
                if self.eat(b'=') {
                    if self.eat(b'=') { BangEqEq } else { BangEq }
                } else {
                    self.error(
                        start,
                        "unexpected character `!` (did you mean `!=` or `not`?)",
                    );
                    Error
                }
            }
            b':' => {
                if self.eat(b':') {
                    if self.eat(b'>') {
                        ColonColonGt
                    } else {
                        ColonColon
                    }
                } else if self.eat(b'>') {
                    if self.eat(b'>') { ColonGtGt } else { ColonGt }
                } else if self.eat(b'=') {
                    ColonEq
                } else {
                    Colon
                }
            }
            b'<' => {
                if self.eat(b'=') {
                    LtEq
                } else {
                    Lt
                }
            }
            b'>' => {
                if self.eat(b'=') {
                    GtEq
                } else {
                    Gt
                }
            }
            b'+' => Plus,
            b'-' => {
                if self.eat(b'>') {
                    Arrow
                } else {
                    Minus
                }
            }
            b'*' => {
                if self.eat(b'*') {
                    StarStar
                } else {
                    Star
                }
            }
            b'%' => Percent,
            b'^' => Caret,
            b'|' => Pipe,
            b'&' => Amp,
            b'@' => {
                if self.eat(b'@') {
                    AtAt
                } else {
                    At
                }
            }
            b'#' => Hash,
            b'$' => Dollar,
            b'(' => LParen,
            b')' => RParen,
            b'[' => LBracket,
            b']' => RBracket,
            b'{' => LBrace,
            b'}' => RBrace,

            _ => {
                // Re-sync to a UTF-8 character boundary so the Error token
                // covers the whole character.
                while self.pos < self.src.len() && !self.text.is_char_boundary(self.pos) {
                    self.pos += 1;
                }
                let ch = self.span_from(start).slice(self.text);
                self.error(start, format!("unexpected character `{ch}`"));
                Error
            }
        }
    }

    /// DECIMAL_VALUE or EXP_VALUE. The exponent is consumed only if complete
    /// (`1e` lexes as `1` then ident `e`, matching the grammar's terminals).
    fn number(&mut self) -> TokenKind {
        while matches!(self.peek(0), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if matches!(self.peek(0), Some(b'e' | b'E')) {
            let mut ahead = 1;
            if matches!(self.peek(1), Some(b'+' | b'-')) {
                ahead = 2;
            }
            if matches!(self.peek(ahead), Some(b'0'..=b'9')) {
                self.pos += ahead + 1;
                while matches!(self.peek(0), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
                return TokenKind::Exp;
            }
        }
        TokenKind::Decimal
    }

    /// Body of UNRESTRICTED_NAME (`quote == '\''`) or STRING_VALUE
    /// (`quote == '"'`). Escapes are validated here; use [`unescape`] to get
    /// the decoded value.
    fn quoted(&mut self, start: usize, quote: u8, what: &str) -> TokenKind {
        loop {
            match self.peek(0) {
                None => {
                    self.error(start, format!("unterminated {what}"));
                    return TokenKind::Error;
                }
                Some(b'\\') => {
                    self.pos += 1;
                    match self.peek(0) {
                        Some(b) if escaped_character(char::from(b)).is_some() => {
                            self.pos += 1;
                        }
                        Some(b) => {
                            let esc_start = self.pos - 1;
                            // Skip the whole (possibly multi-byte) escaped
                            // character so the error span stays on a char
                            // boundary.
                            self.pos += utf8_len(b);
                            let esc = self.span_from(esc_start).slice(self.text);
                            self.error(esc_start, format!("invalid escape sequence `{esc}`"));
                        }
                        None => {}
                    }
                }
                Some(b) if b == quote => {
                    self.pos += 1;
                    return if quote == b'\'' {
                        TokenKind::UnrestrictedName
                    } else {
                        TokenKind::String
                    };
                }
                Some(_) => {
                    let character_start = self.pos;
                    let character = self.text[self.pos..].chars().next().unwrap();
                    self.pos += character.len_utf8();
                    if character.is_control() {
                        self.error(
                            character_start,
                            format!(
                                "unescaped control character U+{:04X} in {what}",
                                u32::from(character)
                            ),
                        );
                    }
                }
            }
        }
    }

    /// Consume up to and including the next `*/`. Returns false at EOF.
    fn eat_until_star_slash(&mut self) -> bool {
        while self.pos < self.src.len() {
            if self.src[self.pos] == b'*' && self.peek(1) == Some(b'/') {
                self.pos += 2;
                return true;
            }
            self.pos += 1;
        }
        false
    }
}

/// Decode the value of an `UNRESTRICTED_NAME` or `STRING_VALUE` token: strip
/// the quotes and resolve escape sequences. Invalid escapes (already reported
/// by the lexer) are kept verbatim without the backslash.
///
/// Total over any input: a leading quote and a trailing quote are removed
/// when present, so text that is not a complete token — empty, unterminated,
/// or never quoted at all — decodes to what it holds rather than failing.
#[must_use]
pub fn unescape(raw: &str) -> String {
    let inner = raw.strip_prefix(['\'', '"']).unwrap_or(raw);
    let inner = inner.strip_suffix(['\'', '"']).unwrap_or(inner);
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some(c) => out.push(escaped_character(c).unwrap_or(c)),
            None => {}
        }
    }
    out
}

/// KerML 8.2.2.3 Table 4. Validation and decoding share this exact set.
/// LF is the implementation's decoded newline, independent of the host OS.
fn escaped_character(character: char) -> Option<char> {
    Some(match character {
        'b' => '\u{0008}',
        't' => '\t',
        'n' => '\n',
        'f' => '\u{000C}',
        '"' | '\'' | '\\' => character,
        _ => return None,
    })
}

/// UTF-8 sequence length from a leading byte (continuation bytes yield 1,
/// which keeps forward progress on malformed input).
fn utf8_len(b: u8) -> usize {
    match b {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn covered(source: &str, tokens: &[Token]) {
        let mut end = 0;
        for token in tokens {
            assert_eq!(token.span.start as usize, end);
            end = token.span.end as usize;
            assert!(source.is_char_boundary(token.span.start as usize));
            assert!(source.is_char_boundary(end));
        }
        assert_eq!(end, source.len());
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn form_feed_is_whitespace_but_does_not_terminate_a_line_note() {
        let source = "a \t\u{000c}\r\nb";
        let (tokens, diagnostics) = tokenize(source);
        assert!(diagnostics.is_empty());
        assert_eq!(
            tokens.iter().map(|t| t.kind).collect::<Vec<_>>(),
            [
                TokenKind::Ident,
                TokenKind::Whitespace,
                TokenKind::Ident,
                TokenKind::Eof
            ]
        );
        assert_eq!(tokens[1].span.slice(source), " \t\u{000c}\r\n");
        covered(source, &tokens);
        let source = "// note\u{000c}still note\nnext /*\t\u{0085}\n*/";
        let (tokens, diagnostics) = tokenize(source);
        assert!(diagnostics.is_empty());
        assert_eq!(tokens[0].kind, TokenKind::LineNote);
        assert_eq!(tokens[0].span.slice(source), "// note\u{000c}still note");
        covered(source, &tokens);
    }

    #[test]
    fn all_table_four_escapes_decode_in_names_and_strings() {
        for quote in ['\'', '"'] {
            for (escape, value) in [
                ('b', '\u{0008}'),
                ('f', '\u{000c}'),
                ('t', '\t'),
                ('n', '\n'),
                ('\'', '\''),
                ('"', '"'),
                ('\\', '\\'),
            ] {
                let source = format!("{quote}海🚢\\{escape}{quote}");
                let (tokens, diagnostics) = tokenize(&source);
                assert!(diagnostics.is_empty(), "{diagnostics:?}");
                assert_eq!(
                    tokens[0].kind,
                    if quote == '\'' {
                        TokenKind::UnrestrictedName
                    } else {
                        TokenKind::String
                    }
                );
                assert_eq!(unescape(&source), format!("海🚢{value}"));
                covered(&source, &tokens);
            }
        }
    }

    #[test]
    fn every_raw_c0_and_c1_control_has_an_exact_utf8_span() {
        for quote in ['\'', '"'] {
            for code in (0..=0x1f).chain(0x7f..=0x9f) {
                let character = char::from_u32(code).unwrap();
                let source = format!("{quote}海🚢{character}z{quote} next");
                let (tokens, diagnostics) = tokenize(&source);
                assert_eq!(diagnostics.len(), 1, "U+{code:04X}");
                assert!(diagnostics[0].message.contains(&format!("U+{code:04X}")));
                assert_eq!(diagnostics[0].span.slice(&source), character.to_string());
                assert_eq!(tokens[2].span.slice(&source), "next");
                covered(&source, &tokens);
            }
        }
    }

    #[test]
    fn nonstandard_escapes_are_diagnosed_without_splitting_unicode() {
        for quote in ['\'', '"'] {
            for escape in ['r', 'v', 'a', '0', 'u', 'x', '/', '海', '🚢'] {
                let source = format!("{quote}α\\{escape}{quote} next");
                let (tokens, diagnostics) = tokenize(&source);
                assert_eq!(diagnostics.len(), 1, "{source}");
                assert!(diagnostics[0].message.contains("invalid escape sequence"));
                assert_eq!(diagnostics[0].span.slice(&source), format!("\\{escape}"));
                assert_eq!(
                    unescape(tokens[0].span.slice(&source)),
                    format!("α{escape}")
                );
                covered(&source, &tokens);
            }
        }
    }

    #[test]
    fn unterminated_quotes_and_trailing_escapes_preserve_complete_coverage() {
        for quote in ['\'', '"'] {
            for suffix in ["", "\\", "\\海", "\u{0085}"] {
                let source = format!("{quote}海🚢{suffix}");
                let (tokens, diagnostics) = tokenize(&source);
                assert_eq!(tokens[0].kind, TokenKind::Error);
                assert!(diagnostics
                    .iter()
                    .any(|d| d.message.contains("unterminated")));
                for diagnostic in diagnostics {
                    assert!(source.is_char_boundary(diagnostic.span.start as usize));
                    assert!(source.is_char_boundary(diagnostic.span.end as usize));
                }
                covered(&source, &tokens);
            }
        }
    }
}

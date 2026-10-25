//! BAF script syntax highlighting for the BCS viewer.
//!
//! [`tokenizer`] turns `&str` into
//! [`Token`]s with byte spans and a [`TokenKind`]. It is context-aware:
//! identifiers followed by `(` are **triggers** inside `IF … THEN`,
//! **actions** inside `THEN … END`, and plain identifiers elsewhere, which
//! mirrors NearInfinity's BCS editor without TRIGGER.IDS / ACTION.IDS
//! lookups. [`lines`] splits the tokens into the viewer's rows of spans.

pub use tokenizer::TokenKind;
use tokenizer::tokenize;

/// How many spaces a tab stands for.
const TAB_WIDTH: usize = 4;

/// A run of one token kind within a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub kind: TokenKind,
    pub text: String,
}

/// `text`'s lines as spans of one token kind each, tabs expanded.
///
/// Whitespace is folded into the token that follows it on the same line, and
/// dropped at the end of a line: each span is drawn as its own text, and a
/// text's trailing spaces don't count in its width.
pub fn lines(text: &str) -> Vec<Vec<Span>> {
    let mut lines = vec![Vec::new()];
    let mut pending = String::new();
    for token in tokenize(text) {
        let slice = &text[token.start..token.end];
        for (i, piece) in slice.split('\n').enumerate() {
            if i > 0 {
                lines.push(Vec::new());
                pending.clear();
            }
            let piece = piece
                .trim_end_matches('\r')
                .replace('\t', &" ".repeat(TAB_WIDTH));
            if token.kind == TokenKind::Whitespace {
                pending.push_str(&piece);
            } else if !piece.is_empty() {
                let text = std::mem::take(&mut pending) + &piece;
                if let Some(line) = lines.last_mut() {
                    line.push(Span {
                        kind: token.kind,
                        text,
                    });
                }
            }
        }
    }
    if text.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// A line's width, in characters.
pub fn line_width(line: &[Span]) -> usize {
    line.iter().map(|span| span.text.chars().count()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(kind: TokenKind, text: &str) -> Span {
        Span {
            kind,
            text: text.to_owned(),
        }
    }

    #[test]
    fn lines_fold_whitespace_into_the_next_token() {
        let lines = lines("IF\n    HPPercentLT(Myself,50)\nTHEN\n\tWait(3)  \nEND\n");
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0], [span(TokenKind::Keyword, "IF")]);
        assert_eq!(lines[1][0], span(TokenKind::Trigger, "    HPPercentLT"));
        assert_eq!(lines[1][3], span(TokenKind::Punctuation, ","));
        assert_eq!(lines[3][0], span(TokenKind::Action, "    Wait"));
        assert_eq!(lines[3].last(), Some(&span(TokenKind::Punctuation, ")")));
        assert_eq!(line_width(&lines[1]), "    HPPercentLT(Myself,50)".len());
    }

    #[test]
    fn blank_lines_stay_as_empty_rows() {
        let lines = lines("A\n\n  \nB");
        assert_eq!(lines.len(), 4);
        assert!(lines[1].is_empty() && lines[2].is_empty());
        assert_eq!(lines[3], [span(TokenKind::Identifier, "B")]);
    }

    #[test]
    fn crlf_line_ends_are_dropped() {
        assert_eq!(lines("END\r\n"), [[span(TokenKind::Keyword, "END")]]);
    }
}

// ── Tokenizer ───────────────────────────────────────────────────────────────

pub mod tokenizer {
    /// Lexical category of a BAF token.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TokenKind {
        Keyword,
        Trigger,
        Action,
        Identifier,
        String,
        Number,
        Comment,
        Punctuation,
        Whitespace,
    }

    /// Half-open byte range into the source string, plus its kind.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Token {
        pub kind: TokenKind,
        pub start: usize,
        pub end: usize,
    }

    /// Which BAF block we are currently inside. Drives the
    /// trigger-vs-action colouring decision: a function-call identifier
    /// gets the trigger or action kind depending on this state.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Section {
        /// Outside any `IF … END` block.
        Outside,
        /// After `IF`, before the matching `THEN`.
        Condition,
        /// After `THEN`, before the matching `END`.
        Response,
    }

    /// Tokenize a BAF script. Whitespace is preserved as `Whitespace`
    /// tokens so that joining the slices reproduces the original source.
    pub fn tokenize(text: &str) -> Vec<Token> {
        // BAF is ASCII in practice; operating on bytes keeps the scanner
        // simple. We still slice the original `&str` for spans so any
        // multibyte content survives intact (we just won't recognise it
        // as part of an identifier — it falls through as Punctuation).
        let bytes = text.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        let mut section = Section::Outside;

        while i < bytes.len() {
            let start = i;
            let b = bytes[i];
            let token = match b {
                b' ' | b'\t' | b'\n' | b'\r' => {
                    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r') {
                        i += 1;
                    }
                    Token {
                        kind: TokenKind::Whitespace,
                        start,
                        end: i,
                    }
                }
                b'/' if bytes.get(i + 1) == Some(&b'/') => {
                    // `// …` to end of line (or EOF).
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                    Token {
                        kind: TokenKind::Comment,
                        start,
                        end: i,
                    }
                }
                b'"' => {
                    // Quoted string — no escape handling (BAF doesn't use
                    // any). Newlines also terminate, defensively.
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'"' && bytes[i] != b'\n' {
                        i += 1;
                    }
                    if i < bytes.len() && bytes[i] == b'"' {
                        i += 1;
                    }
                    Token {
                        kind: TokenKind::String,
                        start,
                        end: i,
                    }
                }
                b'-' if matches!(bytes.get(i + 1), Some(b'0'..=b'9')) => {
                    i += 2;
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                    Token {
                        kind: TokenKind::Number,
                        start,
                        end: i,
                    }
                }
                b'0'..=b'9' => {
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                    Token {
                        kind: TokenKind::Number,
                        start,
                        end: i,
                    }
                }
                b if is_ident_start(b) => {
                    while i < bytes.len() && is_ident_cont(bytes[i]) {
                        i += 1;
                    }
                    let word = &text[start..i];
                    let kind = match word {
                        "IF" => {
                            section = Section::Condition;
                            TokenKind::Keyword
                        }
                        "THEN" => {
                            section = Section::Response;
                            TokenKind::Keyword
                        }
                        "END" => {
                            section = Section::Outside;
                            TokenKind::Keyword
                        }
                        "RESPONSE" | "OR" => TokenKind::Keyword,
                        _ => {
                            // Function call vs identifier: skip horizontal
                            // whitespace and check for `(`. Identifiers in
                            // call position get coloured by the current
                            // section so the IF/RESPONSE flow stands out.
                            let mut j = i;
                            while j < bytes.len() && matches!(bytes[j], b' ' | b'\t') {
                                j += 1;
                            }
                            let is_call = j < bytes.len() && bytes[j] == b'(';
                            if is_call {
                                match section {
                                    Section::Condition => TokenKind::Trigger,
                                    Section::Response => TokenKind::Action,
                                    Section::Outside => TokenKind::Identifier,
                                }
                            } else {
                                TokenKind::Identifier
                            }
                        }
                    };
                    Token {
                        kind,
                        start,
                        end: i,
                    }
                }
                _ => {
                    // Single-byte fallthrough: punctuation, or any byte
                    // we don't recognise. Advance one byte; multibyte
                    // UTF-8 sequences will land entirely in this branch,
                    // one byte at a time, but rendering treats them as
                    // contiguous Punctuation runs which is fine.
                    i += 1;
                    Token {
                        kind: TokenKind::Punctuation,
                        start,
                        end: i,
                    }
                }
            };
            tokens.push(token);
        }

        tokens
    }

    fn is_ident_start(b: u8) -> bool {
        b.is_ascii_alphabetic() || b == b'_'
    }

    fn is_ident_cont(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Helper: return `(kind, slice)` pairs for every token in `text`.
        fn kinds_and_slices(text: &str) -> Vec<(TokenKind, &str)> {
            tokenize(text)
                .into_iter()
                .map(|t| (t.kind, &text[t.start..t.end]))
                .collect()
        }

        #[test]
        fn empty_input_yields_no_tokens() {
            assert!(tokenize("").is_empty());
        }

        #[test]
        fn whitespace_is_preserved_as_a_run() {
            assert_eq!(
                kinds_and_slices("  \n\t"),
                vec![(TokenKind::Whitespace, "  \n\t")]
            );
        }

        #[test]
        fn keywords_recognised() {
            let toks = kinds_and_slices("IF THEN RESPONSE END OR");
            let kinds: Vec<TokenKind> = toks
                .iter()
                .filter(|(k, _)| *k != TokenKind::Whitespace)
                .map(|(k, _)| *k)
                .collect();
            assert_eq!(
                kinds,
                vec![
                    TokenKind::Keyword,
                    TokenKind::Keyword,
                    TokenKind::Keyword,
                    TokenKind::Keyword,
                    TokenKind::Keyword,
                ]
            );
        }

        #[test]
        fn comment_runs_to_end_of_line() {
            let toks = kinds_and_slices("Foo // bar\nBaz");
            // ["Foo", " ", "// bar", "\n", "Baz"]
            assert_eq!(toks[0], (TokenKind::Identifier, "Foo"));
            assert_eq!(toks[2], (TokenKind::Comment, "// bar"));
            assert_eq!(toks[4], (TokenKind::Identifier, "Baz"));
        }

        #[test]
        fn string_literal() {
            assert_eq!(
                kinds_and_slices(r#""hello world""#),
                vec![(TokenKind::String, "\"hello world\"")]
            );
        }

        #[test]
        fn unterminated_string_still_tokenises() {
            // Defensive: the lexer must not panic on a missing closing quote.
            let toks = kinds_and_slices("\"oops\nrest");
            assert_eq!(toks[0].0, TokenKind::String);
        }

        #[test]
        fn integers_signed_and_unsigned() {
            assert_eq!(
                kinds_and_slices("42 -7 100"),
                vec![
                    (TokenKind::Number, "42"),
                    (TokenKind::Whitespace, " "),
                    (TokenKind::Number, "-7"),
                    (TokenKind::Whitespace, " "),
                    (TokenKind::Number, "100"),
                ]
            );
        }

        #[test]
        fn function_call_in_condition_is_trigger() {
            // The `IF` keyword switches the section so the following
            // call-position identifier is coloured as a trigger.
            let toks = kinds_and_slices("IF\n    HPPercentLT(Myself,50)\n");
            // Find the `HPPercentLT` token.
            let hp = toks.iter().find(|(_, s)| *s == "HPPercentLT").unwrap();
            assert_eq!(hp.0, TokenKind::Trigger);
            // `Myself` is not in call position → plain identifier.
            let me = toks.iter().find(|(_, s)| *s == "Myself").unwrap();
            assert_eq!(me.0, TokenKind::Identifier);
        }

        #[test]
        fn function_call_in_response_is_action() {
            let toks = kinds_and_slices("IF\nTHEN\n    Wait(3)\nEND\n");
            let wait = toks.iter().find(|(_, s)| *s == "Wait").unwrap();
            assert_eq!(wait.0, TokenKind::Action);
        }

        #[test]
        fn function_call_outside_blocks_is_identifier() {
            // Without an enclosing IF/RESPONSE we don't know whether a
            // call is a trigger or action; fall back to plain identifier.
            let toks = kinds_and_slices("Wait(3)");
            assert_eq!(toks[0], (TokenKind::Identifier, "Wait"));
        }

        #[test]
        fn end_keyword_resets_section() {
            // After `END` we are back to the outside section, so the
            // next call-position identifier is not coloured as an action.
            let toks = kinds_and_slices("IF\nTHEN\n    Foo()\nEND\nBar()");
            assert_eq!(
                toks.iter().find(|(_, s)| *s == "Foo").unwrap().0,
                TokenKind::Action
            );
            assert_eq!(
                toks.iter().find(|(_, s)| *s == "Bar").unwrap().0,
                TokenKind::Identifier
            );
        }

        #[test]
        fn punctuation_runs_as_individual_tokens() {
            let toks = kinds_and_slices("!(");
            assert_eq!(toks[0], (TokenKind::Punctuation, "!"));
            assert_eq!(toks[1], (TokenKind::Punctuation, "("));
        }

        #[test]
        fn round_trip_preserves_source() {
            let src = "IF\n    HPPercentLT(Myself,50)\nTHEN\n    Wait(3)\nEND\n";
            let rebuilt: String = tokenize(src)
                .into_iter()
                .map(|t| &src[t.start..t.end])
                .collect();
            assert_eq!(rebuilt, src);
        }
    }
}

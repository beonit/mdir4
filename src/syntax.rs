use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    path::Path,
    sync::OnceLock,
};

use two_face::re_exports::syntect::{
    easy::ScopeRangeIterator,
    parsing::{ParseState, Scope, ScopeStack, SyntaxReference, SyntaxSet},
};

pub const MAX_SYNTAX_BYTES: usize = 256 * 1024;
pub const MAX_SYNTAX_LINE_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    Comment,
    Keyword,
    String,
    Number,
    Type,
    Function,
    Variable,
    Constant,
    Attribute,
    Tag,
    Heading,
    Link,
    Macro,
    Operator,
    Punctuation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxSpan {
    pub start: usize,
    pub end: usize,
    pub kind: SyntaxKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxDocument {
    pub language: String,
    pub lines: Vec<Vec<SyntaxSpan>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlainReason {
    Unsupported,
    TooLarge,
    LineTooLong,
    ParserError,
    InvalidRanges,
    Panicked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighlightOutcome {
    Highlighted(SyntaxDocument),
    Plain(PlainReason),
}

pub fn highlight(path: &Path, lines: &[&str], byte_len: usize) -> HighlightOutcome {
    if byte_len > MAX_SYNTAX_BYTES {
        return HighlightOutcome::Plain(PlainReason::TooLarge);
    }
    if lines.iter().any(|line| line.len() > MAX_SYNTAX_LINE_BYTES) {
        return HighlightOutcome::Plain(PlainReason::LineTooLong);
    }

    guarded_highlight(|| highlight_inner(path, lines))
}

pub fn highlight_diff(path: &Path, lines: &[&str], byte_len: usize) -> HighlightOutcome {
    if byte_len > MAX_SYNTAX_BYTES {
        return HighlightOutcome::Plain(PlainReason::TooLarge);
    }
    if lines.iter().any(|line| line.len() > MAX_SYNTAX_LINE_BYTES) {
        return HighlightOutcome::Plain(PlainReason::LineTooLong);
    }

    guarded_highlight(|| {
        let language = detect_syntax(path, "")
            .filter(|syntax| syntax.name != "Plain Text")
            .map(|syntax| language_label(&syntax.name))
            .ok_or(PlainReason::Unsupported)?;
        let mut highlighted_lines = Vec::with_capacity(lines.len());
        for line in lines {
            let Some(code) = diff_code(line) else {
                highlighted_lines.push(Vec::new());
                continue;
            };
            let SyntaxDocument { lines, .. } = highlight_inner(path, &[code])?;
            let spans = lines.into_iter().next().unwrap_or_default();
            highlighted_lines.push(
                spans
                    .into_iter()
                    .map(|span| SyntaxSpan {
                        start: span.start + 1,
                        end: span.end + 1,
                        kind: span.kind,
                    })
                    .collect(),
            );
        }
        Ok(SyntaxDocument {
            language,
            lines: highlighted_lines,
        })
    })
}

fn highlight_inner(path: &Path, lines: &[&str]) -> Result<SyntaxDocument, PlainReason> {
    let syntax = detect_syntax(path, lines.first().copied().unwrap_or_default())
        .ok_or(PlainReason::Unsupported)?;
    if syntax.name == "Plain Text" {
        return Err(PlainReason::Unsupported);
    }

    let mut parser = ParseState::new(syntax);
    let mut scope_stack = ScopeStack::new();
    let mut highlighted_lines = Vec::with_capacity(lines.len());

    for line in lines {
        let operations = parser
            .parse_line(line, syntax_set())
            .map_err(|_| PlainReason::ParserError)?;
        let mut spans = Vec::new();
        for (range, operation) in ScopeRangeIterator::new(&operations, line) {
            scope_stack
                .apply(operation)
                .map_err(|_| PlainReason::ParserError)?;
            if range.is_empty() {
                continue;
            }
            if !line.is_char_boundary(range.start) || !line.is_char_boundary(range.end) {
                return Err(PlainReason::InvalidRanges);
            }
            if let Some(kind) = classify_scopes(&scope_stack.scopes) {
                spans.push(SyntaxSpan {
                    start: range.start,
                    end: range.end,
                    kind,
                });
            }
        }
        highlighted_lines.push(normalize_spans(spans, line)?);
    }

    Ok(SyntaxDocument {
        language: language_label(&syntax.name),
        lines: highlighted_lines,
    })
}

fn guarded_highlight(
    operation: impl FnOnce() -> Result<SyntaxDocument, PlainReason>,
) -> HighlightOutcome {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(document)) => HighlightOutcome::Highlighted(document),
        Ok(Err(reason)) => HighlightOutcome::Plain(reason),
        Err(_) => HighlightOutcome::Plain(PlainReason::Panicked),
    }
}

fn detect_syntax(path: &Path, first_line: &str) -> Option<&'static SyntaxReference> {
    let syntaxes = syntax_set();
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| syntaxes.find_syntax_by_token(name))
        .or_else(|| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .and_then(|extension| syntaxes.find_syntax_by_extension(extension))
        })
        .or_else(|| syntaxes.find_syntax_by_first_line(first_line))
}

fn syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(two_face::syntax::extra_no_newlines)
}

fn language_label(name: &str) -> String {
    match name {
        "Bourne Again Shell (bash)" => "BASH".to_string(),
        "TypeScript" => "TS".to_string(),
        "Markdown" => "MD".to_string(),
        "Plain Text" => String::new(),
        other => other.to_ascii_uppercase(),
    }
}

fn diff_code(line: &str) -> Option<&str> {
    if line.starts_with("+++ ") || line.starts_with("--- ") {
        return None;
    }
    line.strip_prefix(['+', '-', ' '])
}

fn classify_scopes(scopes: &[Scope]) -> Option<SyntaxKind> {
    semantic_rules().iter().find_map(|(kind, prefixes)| {
        scopes
            .iter()
            .rev()
            .any(|scope| prefixes.iter().any(|prefix| prefix.is_prefix_of(*scope)))
            .then_some(*kind)
    })
}

fn semantic_rules() -> &'static Vec<(SyntaxKind, Vec<Scope>)> {
    static RULES: OnceLock<Vec<(SyntaxKind, Vec<Scope>)>> = OnceLock::new();
    RULES.get_or_init(|| {
        const DEFINITIONS: &[(SyntaxKind, &[&str])] = &[
            (SyntaxKind::Comment, &["comment"]),
            (SyntaxKind::Macro, &["support.macro", "entity.name.macro"]),
            (SyntaxKind::Heading, &["markup.heading"]),
            (
                SyntaxKind::Link,
                &["markup.underline.link", "markup.link", "string.other.link"],
            ),
            (
                SyntaxKind::String,
                &["string", "constant.character", "regexp"],
            ),
            (SyntaxKind::Number, &["constant.numeric"]),
            (
                SyntaxKind::Constant,
                &["constant.language", "constant.other"],
            ),
            (
                SyntaxKind::Type,
                &[
                    "entity.name.type",
                    "support.type",
                    "storage.type",
                    "entity.other.inherited-class",
                ],
            ),
            (
                SyntaxKind::Function,
                &[
                    "entity.name.function",
                    "support.function",
                    "meta.function-call",
                ],
            ),
            (
                SyntaxKind::Attribute,
                &["entity.other.attribute-name", "meta.attribute"],
            ),
            (SyntaxKind::Tag, &["entity.name.tag"]),
            (SyntaxKind::Operator, &["keyword.operator"]),
            (SyntaxKind::Keyword, &["keyword", "storage"]),
            (SyntaxKind::Variable, &["variable", "entity.name.variable"]),
            (SyntaxKind::Punctuation, &["punctuation"]),
        ];

        DEFINITIONS
            .iter()
            .map(|(kind, prefixes)| {
                (
                    *kind,
                    prefixes
                        .iter()
                        .map(|prefix| Scope::new(prefix).expect("static semantic scope is valid"))
                        .collect(),
                )
            })
            .collect()
    })
}

fn normalize_spans(spans: Vec<SyntaxSpan>, line: &str) -> Result<Vec<SyntaxSpan>, PlainReason> {
    let mut normalized: Vec<SyntaxSpan> = Vec::with_capacity(spans.len());
    for span in spans {
        if span.start >= span.end
            || span.end > line.len()
            || !line.is_char_boundary(span.start)
            || !line.is_char_boundary(span.end)
        {
            return Err(PlainReason::InvalidRanges);
        }
        if let Some(previous) = normalized.last_mut() {
            if span.start < previous.end {
                return Err(PlainReason::InvalidRanges);
            }
            if span.start == previous.end && span.kind == previous.kind {
                previous.end = span.end;
                continue;
            }
        }
        normalized.push(span);
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn highlighted(path: &str, lines: &[&str]) -> SyntaxDocument {
        match highlight(
            Path::new(path),
            lines,
            lines.iter().map(|line| line.len()).sum(),
        ) {
            HighlightOutcome::Highlighted(document) => document,
            HighlightOutcome::Plain(reason) => panic!("expected highlighting, got {reason:?}"),
        }
    }

    #[test]
    fn detects_the_supported_first_wave_languages() {
        for (path, source, language) in [
            ("main.rs", "fn main() {}", "RUST"),
            ("Cargo.toml", "[package]", "TOML"),
            ("package.json", "{\"name\": \"mdir4\"}", "JSON"),
            ("README.md", "# Mdir4", "MD"),
            ("script.sh", "#!/usr/bin/env bash", "BASH"),
            ("Dockerfile", "FROM rust:latest", "DOCKERFILE"),
            ("Makefile", "all:", "MAKEFILE"),
            ("app.ts", "const answer: number = 42;", "TS"),
        ] {
            assert_eq!(highlighted(path, &[source]).language, language, "{path}");
        }
    }

    #[test]
    fn detects_extensionless_shebang_files() {
        assert_eq!(
            highlighted("script", &["#!/usr/bin/env python3", "print('ok')"]).language,
            "PYTHON"
        );
    }

    #[test]
    fn unsupported_text_stays_plain() {
        assert_eq!(
            highlight(Path::new("notes.txt"), &["hello"], 5),
            HighlightOutcome::Plain(PlainReason::Unsupported)
        );
    }

    #[test]
    fn rust_source_produces_semantic_spans() {
        let document = highlighted(
            "main.rs",
            &[
                "// comment",
                "fn main() { let answer: u64 = 42; println!(\"hello\"); return; }",
            ],
        );
        assert!(
            document.lines[0]
                .iter()
                .any(|span| span.kind == SyntaxKind::Comment)
        );
        assert!(
            document.lines[1]
                .iter()
                .any(|span| span.kind == SyntaxKind::Keyword)
        );
        assert!(
            document.lines[1]
                .iter()
                .any(|span| span.kind == SyntaxKind::Number)
        );
        assert!(
            document.lines[1]
                .iter()
                .any(|span| span.kind == SyntaxKind::String)
        );
        assert!(
            document.lines[1]
                .iter()
                .any(|span| span.kind == SyntaxKind::Macro)
        );
        assert!(
            document.lines[1]
                .iter()
                .any(|span| span.kind == SyntaxKind::Operator)
        );
    }

    #[test]
    fn diff_lines_highlight_code_after_the_marker() {
        let document = match highlight_diff(
            Path::new("main.rs"),
            &[
                "diff --git a/main.rs b/main.rs",
                "@@ -1 +1 @@",
                "-let old = 1;",
                "+let new = 42;",
                " fn unchanged() {}",
            ],
            88,
        ) {
            HighlightOutcome::Highlighted(document) => document,
            HighlightOutcome::Plain(reason) => panic!("expected highlighted diff, got {reason:?}"),
        };

        assert!(document.lines[0].is_empty());
        assert!(document.lines[1].is_empty());
        assert!(
            document.lines[2]
                .iter()
                .any(|span| span.start > 0 && span.kind == SyntaxKind::Type)
        );
        assert!(
            document.lines[3]
                .iter()
                .any(|span| span.start > 0 && span.kind == SyntaxKind::Number)
        );
    }

    #[test]
    fn multiline_comments_keep_their_semantic_kind() {
        let document = highlighted("main.rs", &["/* opening", "still a comment */"]);
        for spans in document.lines {
            assert!(spans.iter().any(|span| span.kind == SyntaxKind::Comment));
        }
    }

    #[test]
    fn unicode_span_boundaries_remain_valid() {
        let line = "let greeting = \"안녕 👋\"; // 주석";
        let document = highlighted("main.rs", &[line]);
        for span in &document.lines[0] {
            assert!(line.is_char_boundary(span.start));
            assert!(line.is_char_boundary(span.end));
            assert!(span.start < span.end);
            assert!(span.end <= line.len());
        }
    }

    #[test]
    fn malformed_source_never_escapes_the_adapter() {
        for (path, lines) in [
            (
                "main.rs",
                vec!["fn main() {", "    let value = \"unfinished"],
            ),
            ("package.json", vec!["{ \"items\": [1, 2,"]),
            ("Cargo.toml", vec!["[package", "name = \"mdir4\""]),
        ] {
            let outcome = highlight(
                Path::new(path),
                &lines,
                lines.iter().map(|line| line.len()).sum(),
            );
            assert!(matches!(
                outcome,
                HighlightOutcome::Highlighted(_)
                    | HighlightOutcome::Plain(PlainReason::ParserError)
            ));
        }
    }

    #[test]
    fn large_documents_and_long_lines_fall_back_before_parsing() {
        assert_eq!(
            highlight(
                Path::new("main.rs"),
                &["fn main() {}"],
                MAX_SYNTAX_BYTES + 1
            ),
            HighlightOutcome::Plain(PlainReason::TooLarge)
        );
        let long_line = "x".repeat(MAX_SYNTAX_LINE_BYTES + 1);
        assert_eq!(
            highlight(Path::new("main.rs"), &[&long_line], long_line.len()),
            HighlightOutcome::Plain(PlainReason::LineTooLong)
        );
    }

    #[test]
    fn invalid_ranges_are_rejected_and_adjacent_kinds_are_merged() {
        let line = "hello";
        assert_eq!(
            normalize_spans(
                vec![
                    SyntaxSpan {
                        start: 0,
                        end: 2,
                        kind: SyntaxKind::Keyword,
                    },
                    SyntaxSpan {
                        start: 2,
                        end: 5,
                        kind: SyntaxKind::Keyword,
                    },
                ],
                line,
            )
            .unwrap(),
            vec![SyntaxSpan {
                start: 0,
                end: 5,
                kind: SyntaxKind::Keyword,
            }]
        );
        assert_eq!(
            normalize_spans(
                vec![SyntaxSpan {
                    start: 0,
                    end: 6,
                    kind: SyntaxKind::Keyword,
                }],
                line,
            ),
            Err(PlainReason::InvalidRanges)
        );
    }

    #[test]
    fn scope_classification_prefers_comments_over_nested_punctuation() {
        let scopes = [
            Scope::new("source.rust").unwrap(),
            Scope::new("comment.line.double-slash.rust").unwrap(),
            Scope::new("punctuation.definition.comment.rust").unwrap(),
        ];
        assert_eq!(classify_scopes(&scopes), Some(SyntaxKind::Comment));
    }

    #[test]
    fn adapter_errors_and_panics_become_plain_outcomes() {
        assert_eq!(
            guarded_highlight(|| Err(PlainReason::ParserError)),
            HighlightOutcome::Plain(PlainReason::ParserError)
        );
        assert_eq!(
            guarded_highlight(|| -> Result<SyntaxDocument, PlainReason> { panic!("probe") }),
            HighlightOutcome::Plain(PlainReason::Panicked)
        );
    }
}

//! Compiled regexes and numeric constants that drive the heuristic.
//!
//! The patterns mirror the classifier rules. Some are anchored (`^...$`) and
//! must match the whole segment. Others are unanchored and fire when the
//! segment contains any matching character. The doc comment on each item states
//! which.

use regex::Regex;
use std::sync::OnceLock;

/// Average characters per token when no language rule applies.
pub const DEFAULT_CHARS_PER_TOKEN: f64 = 6.0;

/// Segments at or below this UTF-16 length count as one token.
pub const SHORT_TOKEN_THRESHOLD: usize = 3;

/// Whitespace test. Anchored: the whole segment must be whitespace.
pub fn whitespace() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s+$").unwrap())
}

/// CJK test. Unanchored: fires if the segment contains one CJK character.
///
/// The class lists BMP blocks for Han, Kana, Hangul, and related symbols.
/// Hiragana (U+3040 to U+309F) is left out on purpose. Adding it would change
/// the counts.
pub fn cjk() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            "[\u{4E00}-\u{9FFF}\u{3400}-\u{4DBF}\u{3000}-\u{303F}\u{FF00}-\u{FFEF}\
             \u{30A0}-\u{30FF}\u{2E80}-\u{2EFF}\u{31C0}-\u{31EF}\u{3200}-\u{32FF}\
             \u{3300}-\u{33FF}\u{AC00}-\u{D7AF}\u{1100}-\u{11FF}\u{3130}-\u{318F}\
             \u{A960}-\u{A97F}\u{D7B0}-\u{D7FF}]",
        )
        .unwrap()
    })
}

/// Numeric test. Anchored. Matches integers and dot or comma grouped numbers.
/// `\d` is ASCII digits only, matching the source which sets no Unicode flag.
pub fn numeric() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[0-9]+(?:[.,][0-9]+)*$").unwrap())
}

/// Punctuation test. Unanchored: fires if the segment contains one of the
/// listed punctuation characters.
pub fn punctuation() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(PUNCTUATION_CLASS).unwrap())
}

/// Alphanumeric test. Anchored. ASCII letters and digits plus Latin-1 accented
/// letters. The class skips U+00D7 and U+00F7 (the multiplication and division
/// signs) so two gaps appear in the accented ranges.
pub fn alphanumeric() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new("^[a-zA-Z0-9\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{00FF}]+$").unwrap()
    })
}

/// The punctuation character class as a regex fragment.
///
/// Each entry is a literal. The trailing `-` is a literal hyphen. The `\]` is a
/// literal `]`. The `\\` is a literal backslash.
const PUNCTUATION_CLASS: &str = r"[.,!?;(){}\[\]<>:/\\|@#$%^&*+=`~_-]";

/// Splitter that keeps delimiters as separate segments.
///
/// Matches a run of whitespace or a run of punctuation. The caller emits the
/// text between matches and the matched runs in order, then drops empty pieces.
pub fn split_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(&format!(r"(\s+|{PUNCTUATION_CLASS}+)")).unwrap())
}

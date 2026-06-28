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

/// The whitespace character class.
///
/// This is the ECMAScript `\s` set, not the Unicode `White_Space` property.
/// The `regex` crate `\s` follows `White_Space`, which drops U+FEFF and adds
/// U+0085. Both shifts change token counts. The class below lists the exact
/// code points ECMAScript `\s` matches so the splitter and the whitespace test
/// agree with the source semantics.
///
/// Members: U+0009, U+000A, U+000B, U+000C, U+000D, U+0020, U+00A0, U+1680,
/// U+2000 through U+200A, U+2028, U+2029, U+202F, U+205F, U+3000, U+FEFF.
const WHITESPACE_CLASS: &str = "[\u{0009}\u{000A}\u{000B}\u{000C}\u{000D}\u{0020}\u{00A0}\u{1680}\u{2000}-\u{200A}\u{2028}\u{2029}\u{202F}\u{205F}\u{3000}\u{FEFF}]";

/// Whitespace test. Anchored: the whole segment must be whitespace.
pub fn whitespace() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(&format!("^{WHITESPACE_CLASS}+$")).unwrap())
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

/// The punctuation character class as a regex fragment.
///
/// Each entry is a literal. The trailing `-` is a literal hyphen. The `\]` is a
/// literal `]`. The `\\` is a literal backslash.
const PUNCTUATION_CLASS: &str = r"[.,!?;(){}\[\]<>:/\\|@#$%^&*+=`~_-]";

/// Splitter that keeps delimiters as separate segments.
///
/// Matches a run of whitespace or a run of punctuation. The caller emits the
/// text between matches and the matched runs in order, then drops empty pieces.
/// Whitespace uses [`WHITESPACE_CLASS`], the ECMAScript `\s` set, so the split
/// boundaries match the source.
pub fn split_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(&format!(r"({WHITESPACE_CLASS}+|{PUNCTUATION_CLASS}+)")).unwrap())
}

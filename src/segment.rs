//! Per-segment token counting and the option types.

use crate::patterns;
use regex::{Regex, RegexBuilder};

/// A language rule. When `pattern` matches a segment, the segment uses
/// `average_chars_per_token` instead of the default.
#[derive(Clone, Debug)]
pub struct LanguageConfig {
    /// Regex that detects the language. It fires if it matches anywhere in the
    /// segment.
    pub pattern: Regex,
    /// Average characters per token for segments this rule matches.
    pub average_chars_per_token: f64,
}

impl LanguageConfig {
    /// Builds a config from a pattern source and a chars-per-token value.
    ///
    /// The pattern compiles case sensitive. Use [`LanguageConfig::case_insensitive`]
    /// to fold case the way a JavaScript `/i` regex does.
    ///
    /// # Errors
    ///
    /// Returns an error if `pattern` is not a valid regex.
    pub fn new(pattern: &str, average_chars_per_token: f64) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
            average_chars_per_token,
        })
    }

    /// Builds a config whose pattern folds case, matching a `/i` JavaScript regex.
    ///
    /// # Errors
    ///
    /// Returns an error if `pattern` is not a valid regex.
    pub fn case_insensitive(
        pattern: &str,
        average_chars_per_token: f64,
    ) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: RegexBuilder::new(pattern).case_insensitive(true).build()?,
            average_chars_per_token,
        })
    }
}

/// Options for token estimation.
///
/// A `None` field uses the built-in default. A `Some` value is used as given,
/// including `Some(0.0)`. This matches JavaScript destructuring defaults, which
/// apply only when a property is absent.
#[derive(Clone, Debug, Default)]
pub struct TokenEstimationOptions {
    /// Default average characters per token. `None` uses
    /// [`patterns::DEFAULT_CHARS_PER_TOKEN`].
    pub default_chars_per_token: Option<f64>,
    /// Language rules. `None` uses the built-in set of three.
    pub language_configs: Option<Vec<LanguageConfig>>,
}

/// Options for [`crate::split_by_tokens`]. Adds chunk overlap.
#[derive(Clone, Debug, Default)]
pub struct SplitByTokensOptions {
    /// Default average characters per token. `None` uses
    /// [`patterns::DEFAULT_CHARS_PER_TOKEN`].
    pub default_chars_per_token: Option<f64>,
    /// Language rules. `None` uses the built-in set of three.
    pub language_configs: Option<Vec<LanguageConfig>>,
    /// Tokens to repeat between consecutive chunks. `None` means zero.
    pub overlap: Option<i64>,
}

/// The three built-in language rules.
///
/// Order matters. The first rule whose pattern matches wins. German and the
/// Romance set use 3 chars per token. The Slavic set uses 3.5.
pub fn default_language_configs() -> Vec<LanguageConfig> {
    vec![
        LanguageConfig::case_insensitive("[äöüßẞ]", 3.0).unwrap(),
        LanguageConfig::case_insensitive("[éèêëàâîïôûùüÿçœæáíóúñ]", 3.0).unwrap(),
        LanguageConfig::case_insensitive("[ąćęłńóśźżěščřžýůúďťň]", 3.5).unwrap(),
    ]
}

/// UTF-16 code-unit length, the unit JavaScript `String.length` reports.
pub fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// Unicode code-point count, the unit `Array.from(s).length` reports.
pub fn char_count(s: &str) -> usize {
    s.chars().count()
}

/// Counts tokens in one segment.
///
/// Rules apply in order and the first match wins:
/// 1. whitespace -> 0
/// 2. contains CJK -> one token per code point
/// 3. whole-segment numeric -> 1
/// 4. UTF-16 length <= 3 -> 1
/// 5. contains punctuation -> ceil(len / 2) when len > 1, else 1
/// 6. whole-segment alphanumeric -> `ceil(len / chars_per_token)`
/// 7. fallback (mixed content) -> `ceil(len / chars_per_token)`
pub fn estimate_segment_tokens(
    segment: &str,
    language_configs: &[LanguageConfig],
    default_chars_per_token: f64,
) -> i64 {
    if patterns::whitespace().is_match(segment) {
        return 0;
    }

    if patterns::cjk().is_match(segment) {
        return char_count(segment) as i64;
    }

    if patterns::numeric().is_match(segment) {
        return 1;
    }

    let len = utf16_len(segment);
    if len <= patterns::SHORT_TOKEN_THRESHOLD {
        return 1;
    }

    if patterns::punctuation().is_match(segment) {
        return if len > 1 {
            (len as f64 / 2.0).ceil() as i64
        } else {
            1
        };
    }

    // Rules 6 and 7 (whole-segment alphanumeric, mixed-content fallback) compute
    // the same value, so the alphanumeric test does not change the result and is
    // not run.
    let chars_per_token = language_specific_chars_per_token(segment, language_configs)
        .unwrap_or(default_chars_per_token);
    (len as f64 / chars_per_token).ceil() as i64
}

/// Returns the chars-per-token of the first matching language rule, or `None`.
fn language_specific_chars_per_token(
    segment: &str,
    language_configs: &[LanguageConfig],
) -> Option<f64> {
    language_configs
        .iter()
        .find(|c| c.pattern.is_match(segment))
        .map(|c| c.average_chars_per_token)
}

/// Splits text into segments, keeping whitespace and punctuation runs as their
/// own segments and dropping empty pieces.
///
/// This reproduces JavaScript `String.split` with a single capturing group:
/// delimiters stay in the output, and the empty strings that fall between
/// adjacent delimiters are removed.
pub fn split_segments(text: &str) -> Vec<&str> {
    let re = patterns::split_pattern();
    let mut out = Vec::new();
    let mut last = 0;
    for m in re.find_iter(text) {
        if m.start() > last {
            out.push(&text[last..m.start()]);
        }
        out.push(m.as_str());
        last = m.end();
    }
    if last < text.len() {
        out.push(&text[last..]);
    }
    out.retain(|s| !s.is_empty());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn est(segment: &str) -> i64 {
        estimate_segment_tokens(
            segment,
            &default_language_configs(),
            patterns::DEFAULT_CHARS_PER_TOKEN,
        )
    }

    #[test]
    fn segment_math_table() {
        // Single-segment inputs and their direct token counts. Values match the
        // classifier rules in order.
        let cases: &[(&str, i64)] = &[
            ("is", 1),        // short token (UTF-16 length <= 3)
            ("a", 1),         // short token
            (",", 1),         // single punctuation, caught by short-token rule
            ("!!!", 1),       // length 3, short-token rule wins before punctuation
            ("!!!!", 2),      // length 4 punctuation, ceil(4 / 2)
            ("::::", 2),      // length 4 punctuation, ceil(4 / 2)
            ("12345", 1),     // numeric
            ("3.14", 1),      // numeric with a dot
            ("1,000.50", 1),  // numeric with separators
            ("Hello", 1),     // alphanumeric, ceil(5 / 6)
            ("sentence", 2),  // alphanumeric, ceil(8 / 6)
            ("道德經", 3),    // CJK, one token per code point
            ("pünktlich", 3), // German rule, ceil(9 / 3)
            ("français", 3),  // Romance rule, ceil(8 / 3)
            ("señor", 2),     // Romance rule, ceil(5 / 3)
            ("źdźbło", 2),    // Slavic rule, ceil(6 / 3.5)
            ("příliš", 2),    // Slavic rule, ceil(6 / 3.5)
        ];
        for (input, want) in cases {
            assert_eq!(est(input), *want, "segment {input:?}");
        }
    }

    #[test]
    fn whitespace_segment_is_zero() {
        assert_eq!(est("   "), 0);
        assert_eq!(est("\t\n "), 0);
    }

    #[test]
    fn uppercase_accents_match_language_rules() {
        // The default rules fold case, so uppercase accented letters match.
        // "ÄÖÜ" is length 3, so it hits the short-token rule and counts as 1.
        // A longer German word proves the rule fires.
        assert_eq!(est("ÜBERGROSS"), (9.0_f64 / 3.0).ceil() as i64);
    }

    #[test]
    fn split_keeps_delimiters() {
        let segs = split_segments("Hello, world!");
        assert_eq!(segs, vec!["Hello", ",", " ", "world", "!"]);
    }

    #[test]
    fn split_drops_empty_pieces() {
        // Adjacent delimiters and leading or trailing delimiters produce no
        // empty segments.
        let segs = split_segments("  a  ");
        assert_eq!(segs, vec!["  ", "a", "  "]);
    }
}

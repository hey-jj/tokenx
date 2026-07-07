//! Token estimation parity, mirroring the estimateTokenCount suite.

mod common;

use common::{fixture, ENGLISH_TEXT, GERMAN_TEXT};
use tokenx::{estimate_token_count, estimate_token_count_with, TokenEstimationOptions};

#[test]
fn short_english_text() {
    assert_eq!(estimate_token_count(ENGLISH_TEXT), 11);
}

#[test]
fn german_text_with_umlauts() {
    assert_eq!(estimate_token_count(GERMAN_TEXT), 49);
}

#[test]
fn fixtures_are_byte_for_byte() {
    // Byte and code-point counts pin the fixtures. A normalized BOM or rewritten
    // line ending would change these and shift the golden token counts.
    let cases: &[(&str, usize, usize)] = &[
        ("pg5200.txt", 142_082, 140_555),
        ("pg22367.txt", 146_052, 143_340),
        ("pg7337.txt", 41_567, 27_531),
        ("pg1982.txt", 38_514, 26_400),
    ];
    for (name, bytes, code_points) in cases {
        let text = fixture(name);
        assert_eq!(text.len(), *bytes, "byte length of {name}");
        assert_eq!(text.chars().count(), *code_points, "code points of {name}");
    }
}

#[test]
fn english_ebook() {
    assert_eq!(estimate_token_count(&fixture("pg5200.txt")), 32_323);
}

#[test]
fn german_ebook() {
    assert_eq!(estimate_token_count(&fixture("pg22367.txt")), 33_968);
}

#[test]
fn chinese_ebook() {
    assert_eq!(estimate_token_count(&fixture("pg7337.txt")), 11_425);
}

#[test]
fn japanese_ebook() {
    // Not in the original assertion set. Exercises the Katakana and Kanji ranges
    // and the absent Hiragana range. The value comes from the heuristic.
    assert_eq!(estimate_token_count(&fixture("pg1982.txt")), 10_533);
}

#[test]
fn empty_input_is_zero() {
    assert_eq!(estimate_token_count(""), 0);
}

#[test]
fn plain_number_stays_one_token() {
    assert_eq!(estimate_token_count("12345"), 1);
}

#[test]
fn decimal_number_stays_one_token() {
    assert_eq!(estimate_token_count("3.14"), 1);
}

#[test]
fn grouped_decimal_number_stays_one_token() {
    assert_eq!(estimate_token_count("1,000.50"), 1);
}

#[test]
fn embedded_numeric_separators_keep_existing_counts() {
    assert_eq!(estimate_token_count("abc1,000def"), 3);
    assert_eq!(estimate_token_count("abc1,000"), 3);
    assert_eq!(estimate_token_count("1,000def"), 3);
}

#[test]
fn failed_long_numeric_candidate_keeps_existing_count() {
    let mut input = String::from("1");
    for _ in 0..16_000 {
        input.push_str(",1");
    }
    input.push('x');

    assert_eq!(estimate_token_count(&input), 32_001);
}

#[test]
fn mixed_content_not_one_to_one() {
    // Regression: URLs and code must use the chars-per-token heuristic, not one
    // token per character.
    let url = "https://example.com/path/to/resource";
    assert!((estimate_token_count(url) as usize) < url.chars().count() / 2);
    assert_eq!(estimate_token_count(url), 13);
}

#[test]
fn ecmascript_whitespace_set() {
    // The whitespace class is the ECMAScript `\s` set, not Unicode White_Space.
    // The two disagree on U+FEFF and U+0085, in opposite directions.

    // U+FEFF (BOM) is ECMAScript whitespace, so it counts 0.
    assert_eq!(estimate_token_count("\u{FEFF}"), 0);
    assert_eq!(estimate_token_count("\u{FEFF}\u{FEFF}"), 0);
    // A BOM between words is a delimiter. "ab" and "cd" each count 1.
    assert_eq!(estimate_token_count("ab\u{FEFF}cd"), 2);

    // U+0085 (NEL) is not ECMAScript whitespace, so it does not split or zero out.
    // "\u{0085}" alone is one short non-whitespace segment.
    assert_eq!(estimate_token_count("\u{0085}"), 1);
    // "ab\u{0085}cd" stays one 5-unit segment, fallback ceil(5 / 6) = 1.
    assert_eq!(estimate_token_count("ab\u{0085}cd"), 1);
}

#[test]
fn mid_text_bom_splits_like_javascript() {
    // A BOM glued to a leading word rides along and the count is unaffected, the
    // case the fixtures hit. A BOM in the middle of text must act as a delimiter.
    // "word\u{FEFF}word" -> ["word", "word"], each ceil(4 / 6) = 1, total 2.
    assert_eq!(estimate_token_count("word\u{FEFF}word"), 2);
    // Without the fix this stays one 9-unit segment and counts ceil(9 / 6) = 2 by
    // coincidence, so use an input where the totals differ. "longword\u{FEFF}x":
    // split -> ceil(8 / 6) + 1 = 2 + 1 = 3, glued -> ceil(10 / 6) = 2.
    assert_eq!(estimate_token_count("longword\u{FEFF}x"), 3);
}

#[test]
fn custom_chars_per_token_increases_count() {
    let default_count = estimate_token_count("Hello world");
    let opts = TokenEstimationOptions {
        default_chars_per_token: Some(4.0),
        ..Default::default()
    };
    let custom = estimate_token_count_with("Hello world", &opts);
    assert!(custom > default_count);
    // Exact pins for both directions.
    assert_eq!(default_count, 2);
    assert_eq!(custom, 4);
}

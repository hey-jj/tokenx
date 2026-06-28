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
    assert_eq!(estimate_token_count(&fixture("pg5200.txt")), 32_325);
}

#[test]
fn german_ebook() {
    assert_eq!(estimate_token_count(&fixture("pg22367.txt")), 33_970);
}

#[test]
fn chinese_ebook() {
    assert_eq!(estimate_token_count(&fixture("pg7337.txt")), 11_427);
}

#[test]
fn japanese_ebook() {
    // Not in the original assertion set. Exercises the Katakana and Kanji ranges
    // and the absent Hiragana range. The value comes from the heuristic.
    assert_eq!(estimate_token_count(&fixture("pg1982.txt")), 10_535);
}

#[test]
fn empty_input_is_zero() {
    assert_eq!(estimate_token_count(""), 0);
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

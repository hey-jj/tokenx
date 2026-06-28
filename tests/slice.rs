//! Token-range slicing parity, mirroring the sliceByTokens suite.

mod common;

use common::{ENGLISH_TEXT, GERMAN_TEXT};
use tokenx::{estimate_token_count, slice_by_tokens, slice_by_tokens_with, TokenEstimationOptions};

#[test]
fn empty_input_and_no_bounds() {
    assert_eq!(slice_by_tokens("", 0, None), "");
    assert_eq!(slice_by_tokens("", 0, Some(5)), "");
    // No bounds returns the entire text.
    assert_eq!(slice_by_tokens(ENGLISH_TEXT, 0, None), ENGLISH_TEXT);
}

#[test]
fn english_positive_indices() {
    let first_two = slice_by_tokens(ENGLISH_TEXT, 0, Some(2));
    let from_third = slice_by_tokens(ENGLISH_TEXT, 2, None);
    assert_eq!(first_two, "Hello,");
    assert_eq!(from_third, " world! This is a short sentence.");
    let combined = first_two.encode_utf16().count() + from_third.encode_utf16().count();
    assert!(combined as f64 > ENGLISH_TEXT.encode_utf16().count() as f64 * 0.8);
}

#[test]
fn german_positive_indices() {
    let first_three = slice_by_tokens(GERMAN_TEXT, 0, Some(3));
    assert_eq!(first_three, "Die pünktl");

    let middle = slice_by_tokens(GERMAN_TEXT, 5, Some(10));
    assert!(!middle.is_empty());
    assert!(middle.chars().count() < GERMAN_TEXT.chars().count());
}

#[test]
fn german_negative_indices() {
    let last_three = slice_by_tokens(GERMAN_TEXT, -3, None);
    assert_eq!(last_three, "lle führen");

    let without_last_two = slice_by_tokens(GERMAN_TEXT, 0, Some(-2));
    assert!(without_last_two.ends_with("Fülle"));

    let middle_negative = slice_by_tokens(GERMAN_TEXT, -8, Some(-3));
    assert!(!middle_negative.is_empty());
    assert!(middle_negative.contains("Hülle"));
}

#[test]
fn edge_cases() {
    // Slice indices are signed, so cast the unsigned count to combine it with
    // offsets.
    let total = estimate_token_count(GERMAN_TEXT) as i64;

    // start at or past end returns empty.
    assert_eq!(slice_by_tokens(GERMAN_TEXT, 10, Some(5)), "");
    assert_eq!(slice_by_tokens(GERMAN_TEXT, 5, Some(5)), "");

    // start past all tokens returns empty.
    assert_eq!(slice_by_tokens(GERMAN_TEXT, total + 10, None), "");
    // end past all tokens slices the whole text.
    assert_eq!(
        slice_by_tokens(GERMAN_TEXT, 0, Some(total + 10)),
        GERMAN_TEXT
    );
    // a large negative start clamps to zero.
    assert_eq!(slice_by_tokens(GERMAN_TEXT, -1000, None), GERMAN_TEXT);
}

#[test]
fn astral_split_emits_replacement_char() {
    // Five U+1F600 emoji, UTF-16 length 10, one word segment, fallback
    // ceil(10 / 6) = 2 tokens. slice_by_tokens(text, 0, Some(1)) cuts at UTF-16
    // unit 5, inside the third surrogate pair. JavaScript keeps a lone high
    // surrogate there. A Rust String cannot, so the lone surrogate becomes
    // U+FFFD. This pins the documented limitation.
    let text = "\u{1F600}\u{1F600}\u{1F600}\u{1F600}\u{1F600}";
    let out = slice_by_tokens(text, 0, Some(1));
    let units: Vec<u16> = out.encode_utf16().collect();
    assert_eq!(units, vec![0xd83d, 0xde00, 0xd83d, 0xde00, 0xfffd]);
    assert!(out.ends_with('\u{FFFD}'));
    // Two whole emoji decode cleanly, the third is the replacement char.
    assert_eq!(out.chars().count(), 3);
}

#[test]
fn slice_with_options_changes_boundaries() {
    // Smaller chars per token raises token counts, so the same token range
    // covers fewer characters.
    assert_eq!(slice_by_tokens(ENGLISH_TEXT, 0, Some(3)), "Hello, world");
    let opts = TokenEstimationOptions {
        default_chars_per_token: Some(2.0),
        ..Default::default()
    };
    assert_eq!(
        slice_by_tokens_with(ENGLISH_TEXT, 0, Some(3), &opts),
        "Hello"
    );
}

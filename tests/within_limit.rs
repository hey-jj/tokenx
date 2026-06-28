//! Token-limit checks, mirroring the isWithinTokenLimit suite.

mod common;

use tokenx::{is_within_token_limit, is_within_token_limit_with, TokenEstimationOptions};

#[test]
fn within_limit() {
    assert!(is_within_token_limit("Short input.", 10));
}

#[test]
fn exceeds_limit() {
    let input =
        "This is a much longer input that should exceed the token limit set for this test case.";
    assert!(!is_within_token_limit(input, 10));
}

#[test]
fn custom_options_change_outcome() {
    // The default estimate stays within 3. With 2 chars per token it exceeds 3.
    assert!(is_within_token_limit("Hello world", 3));
    let opts = TokenEstimationOptions {
        default_chars_per_token: Some(2.0),
        ..Default::default()
    };
    assert!(!is_within_token_limit_with("Hello world", 3, &opts));
}

#[test]
fn limit_is_inclusive() {
    let exact = tokenx::estimate_token_count("Hello, world!");
    assert!(is_within_token_limit("Hello, world!", exact));
    assert!(!is_within_token_limit("Hello, world!", exact - 1));
}

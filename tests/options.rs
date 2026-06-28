//! Option-driven behavior: custom language rules and the deprecated alias.

mod common;

use tokenx::{
    estimate_token_count, estimate_token_count_with, LanguageConfig, TokenEstimationOptions,
};

#[test]
fn custom_language_configs_override_default() {
    // "wonderful" has length 9. The default rule gives ceil(9 / 6) = 2. A custom
    // rule matching the letter "o" with 2 chars per token gives ceil(9 / 2) = 5.
    assert_eq!(estimate_token_count("wonderful"), 2);

    let opts = TokenEstimationOptions {
        language_configs: Some(vec![LanguageConfig::new("o", 2.0).unwrap()]),
        ..Default::default()
    };
    assert_eq!(estimate_token_count_with("wonderful", &opts), 5);
}

#[test]
fn empty_language_configs_falls_back_to_default_chars() {
    // With no language rules, every alphanumeric word uses the default rate.
    let opts = TokenEstimationOptions {
        language_configs: Some(vec![]),
        default_chars_per_token: Some(3.0),
    };
    // "wonderful" length 9, ceil(9 / 3) = 3.
    assert_eq!(estimate_token_count_with("wonderful", &opts), 3);
}

#[test]
fn case_insensitive_config_matches_uppercase() {
    // A case-insensitive rule matches both letter cases.
    let opts = TokenEstimationOptions {
        language_configs: Some(vec![LanguageConfig::case_insensitive("z", 2.0).unwrap()]),
        ..Default::default()
    };
    // "puzzling" length 8, custom rate 2, ceil(8 / 2) = 4.
    assert_eq!(estimate_token_count_with("puzzling", &opts), 4);
    assert_eq!(estimate_token_count_with("PUZZLING", &opts), 4);
}

#[test]
fn zero_chars_per_token_saturates_without_panic() {
    // Some(0.0) is honored, not replaced by the default. A zero rate makes the
    // per-segment division infinite. The count must stay finite and not panic.
    // Each non-trivial segment saturates to u64::MAX, and the saturating sum
    // holds at u64::MAX over multiple segments.
    let opts = TokenEstimationOptions {
        default_chars_per_token: Some(0.0),
        ..Default::default()
    };
    assert_eq!(
        estimate_token_count_with("hello world this is text", &opts),
        u64::MAX
    );
}

#[test]
#[allow(deprecated)]
fn deprecated_alias_matches_estimate() {
    for input in ["", "Hello, world!", "wonderful", "道德經"] {
        assert_eq!(
            tokenx::approximate_token_size(input),
            estimate_token_count(input),
            "alias mismatch for {input:?}"
        );
    }
}

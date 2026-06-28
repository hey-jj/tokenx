//! Chunking parity, mirroring the splitByTokens suite.

mod common;

use common::ENGLISH_TEXT;
use tokenx::{split_by_tokens, split_by_tokens_with, SplitByTokensOptions};

#[test]
fn split_into_chunks_is_lossless() {
    let chunks = split_by_tokens(ENGLISH_TEXT, 5);
    assert!(chunks.len() > 1);
    assert_eq!(chunks.concat(), ENGLISH_TEXT);
    assert_eq!(
        chunks,
        vec!["Hello, world! This", " is a short sentence", "."]
    );
}

#[test]
fn overlap_grows_chunk_count_and_chars() {
    let no_overlap = split_by_tokens(ENGLISH_TEXT, 5);
    let opts = SplitByTokensOptions {
        overlap: Some(2),
        ..Default::default()
    };
    let with_overlap = split_by_tokens_with(ENGLISH_TEXT, 5, &opts);

    assert!(with_overlap.len() >= no_overlap.len());

    let total_with: usize = with_overlap.iter().map(|c| c.encode_utf16().count()).sum();
    let total_without: usize = no_overlap.iter().map(|c| c.encode_utf16().count()).sum();
    assert!(total_with >= total_without);

    assert_eq!(
        with_overlap,
        vec![
            "Hello, world! This",
            "! This is a short",
            "a short sentence.",
            "sentence."
        ]
    );
}

#[test]
fn empty_or_nonpositive_chunk_returns_empty() {
    assert!(split_by_tokens("", 5).is_empty());
    assert!(split_by_tokens("text", 0).is_empty());
    assert!(split_by_tokens("text", -5).is_empty());
}

#[test]
fn short_text_is_single_chunk() {
    assert_eq!(split_by_tokens("Hi there", 100), vec!["Hi there"]);
}

#[test]
fn split_with_options_changes_chunking_but_stays_lossless() {
    let default_chunks = split_by_tokens(ENGLISH_TEXT, 5);
    let opts = SplitByTokensOptions {
        default_chars_per_token: Some(3.0),
        ..Default::default()
    };
    let custom_chunks = split_by_tokens_with(ENGLISH_TEXT, 5, &opts);
    assert_ne!(custom_chunks, default_chunks);
    assert_eq!(custom_chunks.concat(), ENGLISH_TEXT);
    assert_eq!(
        custom_chunks,
        vec!["Hello, world", "! This is a", " short sentence", "."]
    );
}

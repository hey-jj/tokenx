//! Vocab-free heuristic LLM token-count estimation.
//!
//! `tokenx` estimates how many tokens a language model would read in a string.
//! It uses no vocabulary or BPE table. It splits text into segments (words,
//! whitespace runs, punctuation runs), classifies each segment, and assigns a
//! token count per segment with a small set of chars-per-token rules. The result
//! lands within a few percent of a real GPT tokenizer for most text.
//!
//! Four functions make up the API:
//! - [`estimate_token_count`] returns the token estimate for a string.
//! - [`is_within_token_limit`] checks the estimate against a limit.
//! - [`slice_by_tokens`] extracts a token range, like `Array.slice`.
//! - [`split_by_tokens`] chunks text by a token budget, with optional overlap.
//!
//! # Examples
//!
//! ```
//! use tokenx::estimate_token_count;
//!
//! assert_eq!(estimate_token_count("Hello, world! This is a short sentence."), 11);
//! ```
//!
//! Counting operates on UTF-16 code units for lengths and slicing, matching
//! JavaScript string semantics. The CJK rule counts Unicode code points. For
//! text in the Basic Multilingual Plane these agree.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod patterns;
mod segment;

pub use segment::{LanguageConfig, SplitByTokensOptions, TokenEstimationOptions};

use patterns::DEFAULT_CHARS_PER_TOKEN;
use segment::{
    default_language_configs, estimate_segment_tokens, split_segments, utf16_len,
    LanguageConfig as Lc,
};

/// Resolves the chars-per-token and language rules from options.
fn resolve(
    default_chars_per_token: Option<f64>,
    language_configs: Option<&Vec<Lc>>,
) -> (f64, Vec<Lc>) {
    let cpt = default_chars_per_token.unwrap_or(DEFAULT_CHARS_PER_TOKEN);
    let configs = language_configs
        .cloned()
        .unwrap_or_else(default_language_configs);
    (cpt, configs)
}

/// Estimates the number of tokens in `text`.
///
/// Returns 0 for the empty string. The estimate is a non-negative integer.
///
/// # Examples
///
/// ```
/// use tokenx::estimate_token_count;
///
/// assert_eq!(estimate_token_count(""), 0);
/// assert_eq!(estimate_token_count("Hello, world! This is a short sentence."), 11);
/// ```
pub fn estimate_token_count(text: &str) -> i64 {
    estimate_token_count_with(text, &TokenEstimationOptions::default())
}

/// Estimates the number of tokens in `text` with custom options.
///
/// # Examples
///
/// ```
/// use tokenx::{estimate_token_count, estimate_token_count_with, TokenEstimationOptions};
///
/// let opts = TokenEstimationOptions { default_chars_per_token: Some(4.0), ..Default::default() };
/// assert!(estimate_token_count_with("Hello world", &opts) > estimate_token_count("Hello world"));
/// ```
pub fn estimate_token_count_with(text: &str, options: &TokenEstimationOptions) -> i64 {
    if text.is_empty() {
        return 0;
    }
    let (cpt, configs) = resolve(
        options.default_chars_per_token,
        options.language_configs.as_ref(),
    );
    split_segments(text)
        .iter()
        .map(|s| estimate_segment_tokens(s, &configs, cpt))
        .sum()
}

/// Returns the token estimate for `text`. Deprecated alias of
/// [`estimate_token_count`].
#[deprecated(note = "use `estimate_token_count`")]
pub fn approximate_token_size(text: &str) -> i64 {
    estimate_token_count(text)
}

/// Reports whether the estimate for `text` is at or below `token_limit`.
///
/// The comparison is inclusive.
///
/// # Examples
///
/// ```
/// use tokenx::is_within_token_limit;
///
/// assert!(is_within_token_limit("Short input.", 10));
/// ```
pub fn is_within_token_limit(text: &str, token_limit: i64) -> bool {
    is_within_token_limit_with(text, token_limit, &TokenEstimationOptions::default())
}

/// Reports whether the estimate for `text` is at or below `token_limit`, with
/// custom options.
pub fn is_within_token_limit_with(
    text: &str,
    token_limit: i64,
    options: &TokenEstimationOptions,
) -> bool {
    estimate_token_count_with(text, options) <= token_limit
}

/// Extracts the substring covering token positions `[start, end)`.
///
/// Negative `start` or `end` count from the end. `end` of `None` slices to the
/// end of the text. A `start` at or past `end` returns the empty string.
///
/// # Examples
///
/// ```
/// use tokenx::slice_by_tokens;
///
/// let text = "Hello, world! This is a short sentence.";
/// assert_eq!(slice_by_tokens(text, 0, Some(2)), "Hello,");
/// assert_eq!(slice_by_tokens(text, 2, None), " world! This is a short sentence.");
/// ```
pub fn slice_by_tokens(text: &str, start: i64, end: Option<i64>) -> String {
    slice_by_tokens_with(text, start, end, &TokenEstimationOptions::default())
}

/// Extracts a token range with custom options. See [`slice_by_tokens`].
pub fn slice_by_tokens_with(
    text: &str,
    start: i64,
    end: Option<i64>,
    options: &TokenEstimationOptions,
) -> String {
    if text.is_empty() {
        return String::new();
    }
    let (cpt, configs) = resolve(
        options.default_chars_per_token,
        options.language_configs.as_ref(),
    );

    // Total tokens are needed only to resolve negative indices.
    let mut total_tokens = 0i64;
    if start < 0 || end.is_some_and(|e| e < 0) {
        total_tokens = estimate_token_count_with(text, options);
    }

    let normalized_start: f64 = if start < 0 {
        (total_tokens + start).max(0) as f64
    } else {
        start.max(0) as f64
    };
    let normalized_end: f64 = match end {
        None => f64::INFINITY,
        Some(e) if e < 0 => (total_tokens + e).max(0) as f64,
        Some(e) => e as f64,
    };

    if normalized_start >= normalized_end {
        return String::new();
    }

    let mut parts = String::new();
    let mut current_token_pos: f64 = 0.0;
    for seg in split_segments(text) {
        if current_token_pos >= normalized_end {
            break;
        }
        let token_count = estimate_segment_tokens(seg, &configs, cpt);
        let extracted = extract_segment_part(
            seg,
            current_token_pos,
            token_count as f64,
            normalized_start,
            normalized_end,
        );
        if !extracted.is_empty() {
            parts.push_str(&extracted);
        }
        current_token_pos += token_count as f64;
    }
    parts
}

/// Cuts the part of `segment` that falls inside the target token range.
///
/// A zero-token segment (whitespace) is kept whole when its start position lands
/// in `[target_start, target_end)`. A multi-token segment that lands partly in
/// range is cut by linear proportion of its UTF-16 length.
fn extract_segment_part(
    segment: &str,
    segment_token_start: f64,
    segment_token_count: f64,
    target_start: f64,
    target_end: f64,
) -> String {
    if segment_token_count == 0.0 {
        return if segment_token_start >= target_start && segment_token_start < target_end {
            segment.to_string()
        } else {
            String::new()
        };
    }

    let segment_token_end = segment_token_start + segment_token_count;
    if segment_token_start >= target_end || segment_token_end <= target_start {
        return String::new();
    }

    let overlap_start = (target_start - segment_token_start).max(0.0);
    let overlap_end = segment_token_count.min(target_end - segment_token_start);

    if overlap_start == 0.0 && overlap_end == segment_token_count {
        return segment.to_string();
    }

    let len = utf16_len(segment) as f64;
    let char_start = ((overlap_start / segment_token_count) * len).floor() as usize;
    let char_end = ((overlap_end / segment_token_count) * len).ceil() as usize;
    utf16_slice(segment, char_start, char_end)
}

/// Slices `s` on UTF-16 code-unit indices, matching `String.prototype.slice`.
///
/// Indices are clamped to the string length. A start at or past the end yields
/// the empty string.
fn utf16_slice(s: &str, start: usize, end: usize) -> String {
    let units: Vec<u16> = s.encode_utf16().collect();
    let start = start.min(units.len());
    let end = end.min(units.len());
    if start >= end {
        return String::new();
    }
    String::from_utf16_lossy(&units[start..end])
}

/// Splits `text` into chunks, each holding up to `tokens_per_chunk` tokens.
///
/// A chunk can exceed the budget by the last segment added, because a segment is
/// counted after it is placed. `tokens_per_chunk` at or below zero returns an
/// empty vector. With no overlap, joining the chunks reproduces `text`.
///
/// # Examples
///
/// ```
/// use tokenx::split_by_tokens;
///
/// let text = "Hello, world! This is a short sentence.";
/// let chunks = split_by_tokens(text, 5);
/// assert_eq!(chunks.concat(), text);
/// ```
pub fn split_by_tokens(text: &str, tokens_per_chunk: i64) -> Vec<String> {
    split_by_tokens_with(text, tokens_per_chunk, &SplitByTokensOptions::default())
}

/// Splits `text` into chunks with custom options, including overlap. See
/// [`split_by_tokens`].
pub fn split_by_tokens_with(
    text: &str,
    tokens_per_chunk: i64,
    options: &SplitByTokensOptions,
) -> Vec<String> {
    if text.is_empty() || tokens_per_chunk <= 0 {
        return Vec::new();
    }
    let (cpt, configs) = resolve(
        options.default_chars_per_token,
        options.language_configs.as_ref(),
    );
    let overlap = options.overlap.unwrap_or(0);

    let mut chunks: Vec<String> = Vec::new();
    let mut current_chunk: Vec<&str> = Vec::new();
    let mut current_token_count = 0i64;

    for seg in split_segments(text) {
        let token_count = estimate_segment_tokens(seg, &configs, cpt);
        current_chunk.push(seg);
        current_token_count += token_count;

        if current_token_count >= tokens_per_chunk {
            chunks.push(current_chunk.concat());

            if overlap > 0 {
                let mut overlap_segments: Vec<&str> = Vec::new();
                let mut overlap_token_count = 0i64;
                let mut i = current_chunk.len();
                while i > 0 && overlap_token_count < overlap {
                    i -= 1;
                    let sv = current_chunk[i];
                    overlap_token_count += estimate_segment_tokens(sv, &configs, cpt);
                    overlap_segments.insert(0, sv);
                }
                current_chunk = overlap_segments;
                current_token_count = overlap_token_count;
            } else {
                current_chunk.clear();
                current_token_count = 0;
            }
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk.concat());
    }
    chunks
}

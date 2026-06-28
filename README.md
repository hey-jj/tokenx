# tokenx

Vocab-free heuristic LLM token-count estimation.

`tokenx` estimates how many tokens a language model reads in a string. It carries
no vocabulary or BPE table. It splits text into words, whitespace runs, and
punctuation runs, classifies each piece, and assigns tokens with a few
chars-per-token rules. The estimate lands within a few percent of a real GPT
tokenizer for most text, at a fraction of the size and cost.

## Installation

```toml
[dependencies]
tokenx = "0.1"
```

## Usage

```rust
use tokenx::{estimate_token_count, is_within_token_limit, slice_by_tokens, split_by_tokens};

// Count tokens.
assert_eq!(estimate_token_count("Hello, world! This is a short sentence."), 11);

// Check against a budget. The comparison is inclusive.
assert!(is_within_token_limit("Short input.", 10));

// Extract a token range, like Array.slice. Negative indices count from the end.
let text = "Hello, world! This is a short sentence.";
assert_eq!(slice_by_tokens(text, 0, Some(2)), "Hello,");
assert_eq!(slice_by_tokens(text, 2, None), " world! This is a short sentence.");

// Chunk text by a token budget. With no overlap, the chunks rejoin to the input.
let chunks = split_by_tokens(text, 5);
assert_eq!(chunks.concat(), text);
```

## Options

Every function has a `_with` variant that takes options.

```rust
use tokenx::{estimate_token_count_with, LanguageConfig, TokenEstimationOptions};

let opts = TokenEstimationOptions {
    default_chars_per_token: Some(4.0),
    language_configs: Some(vec![LanguageConfig::case_insensitive("[äöü]", 3.0).unwrap()]),
};
let count = estimate_token_count_with("Hallo Welt", &opts);
```

- `default_chars_per_token`: average characters per token when no language rule
  applies. Defaults to 6.
- `language_configs`: ordered rules. The first rule whose pattern matches a
  segment sets that segment's rate. Defaults to built-in rules for German,
  Romance, and Slavic accents.
- `overlap` (split only): tokens to repeat between consecutive chunks. Defaults
  to 0.

## How the estimate works

Each segment is counted by the first matching rule:

1. Whitespace counts as 0 tokens.
2. A segment with any CJK character counts one token per code point.
3. A whole-segment number (`123`, `1,000.50`) counts as 1 token.
4. A segment of 3 characters or fewer counts as 1 token.
5. A punctuation run counts as `ceil(length / 2)` tokens.
6. Anything else counts as `ceil(length / chars_per_token)` tokens.

Lengths use UTF-16 code units, matching JavaScript string semantics. The CJK rule
counts Unicode code points.

## License

Licensed under the [MIT license](LICENSE).

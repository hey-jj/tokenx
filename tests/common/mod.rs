//! Shared fixtures and constants for the integration tests.
//!
//! Each test binary pulls in this module and uses a subset of it, so some items
//! look unused from any single binary.
#![allow(dead_code)]

use std::path::Path;

/// Sentence used across the estimate, slice, and split suites.
pub const ENGLISH_TEXT: &str = "Hello, world! This is a short sentence.";

/// German sentence dense with umlauts. Exercises the German language rule and
/// code-point-correct slicing.
pub const GERMAN_TEXT: &str = "Die pünktlich gewünschte Trüffelfüllung im übergestülpten Würzkümmel-Würfel ist kümmerlich und dürfte fürderhin zu Rüffeln in Hülle und Fülle führen";

/// Reads an ebook fixture as UTF-8, preserving the byte-order mark and CRLF
/// line endings.
pub fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ebooks")
        .join(name);
    std::fs::read_to_string(path).expect("fixture readable as UTF-8")
}

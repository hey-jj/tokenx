# Changelog

## [0.2.0] - 2026-07-07

### Changed
- Standalone decimals and grouped numbers now count as one numeric value, so estimates for inputs such as `3.14` and `1,000.50` decrease. (#13)

### Fixed
- Removed dead branches in the segment splitter, with no change to counts or splits (#15, #16)

### Performance
- Overlapped chunking uses less allocation when building each next chunk. (#14)

### Documentation
- The README now states the exact CJK ranges covered by the one token per code point rule. (#17)

## [0.2.0] - 2026-07-07

### Changed
- Standalone decimals and grouped numbers now count as one numeric value, so estimates for inputs such as `3.14` and `1,000.50` decrease. (#13)

### Fixed
- Removed dead branches in the segment splitter, with no change to counts or splits (#15, #16)

### Performance
- Overlapped chunking uses less allocation when building each next chunk. (#14)

### Documentation
- The README now states the exact CJK ranges covered by the one token per code point rule. (#17)

# Changelog

All notable changes to this project will be documented in this file.

## [0.2.1] - 2026-06-21

### Changed
- Cleaned repository packaging metadata for the polished 0.2 line.
- Kept the public API unchanged from 0.2.0.

## [0.2.0] - 2026-06-21

### Added
- Added `LayoutJob` conversion APIs: `ansi_to_layout_job`,
  `ansi_bytes_to_layout_job`, and `spans_to_layout_job`.
- Added semantic span APIs: `AnsiSpan`, `AnsiStyle`, `AnsiColor`,
  `AnsiIntensity`, `UnderlineStyle`, `ansi_to_spans`, and
  `ansi_bytes_to_spans`.
- Added streaming APIs: `AnsiStreamParser` and `AnsiSpanBuffer`.
- Added `EguiAnsiTheme` with xterm and legacy palettes.
- Added support for underline color, reverse video, hidden text, italic,
  faint/bold intensity, strikethrough, and colon-form RGB SGR parameters.
- Added Criterion benchmarks for one-shot parsing, LayoutJob rendering, stream
  parsing, and stream buffering.

### Changed
- Updated `egui` and `eframe` to `0.34.3`.
- Raised MSRV to Rust `1.92`.
- Replaced regex-based escape scanning with the `vte` parser.
- Updated the demo to the `eframe::App::ui` API.
- Enabled crate-level `missing_docs` and `unsafe_code` lints.
- Kept old `RichText`, `ColoredText`, and `AnsiParser` APIs as compatibility
  entry points.

### Removed
- Removed the runtime dependency on `regex`.

## [0.1.4] - 2025-11-25

### Changed
- chore(Cargo.lock): Update dependency version to 0.1.4
  - Updated project dependencies and initial release
  - Added comprehensive ANSI color parsing support for egui
  - Implemented 4-bit, 8-bit, and 24-bit color models
  - Added examples and documentation

# egui_sgr Architecture

`egui_sgr` is organized around one pipeline:

```text
bytes / str -> vte parser -> ANSI style spans -> egui rendering
```

The crate intentionally does not emulate a terminal screen. It linearizes
visible text, interprets SGR style changes, strips non-rendered control
sequences, and maps the result into egui types.

## API Layers

- `ansi_to_spans` and `ansi_bytes_to_spans` are the semantic parse layer. They
  are useful for tests, custom renderers, and callers that want to cache parsed
  ANSI output independently from an egui theme.
- `spans_to_layout_job` is the egui render layer. It turns semantic spans into
  one `egui::text::LayoutJob`, preserving byte ranges and text layout as a
  single egui widget.
- `ansi_to_layout_job` and `ansi_bytes_to_layout_job` are direct render APIs
  that parse into a `LayoutJob` without allocating an intermediate span list.
- `AnsiStreamParser` is the synchronous streaming parser. It preserves style,
  incomplete escape sequences, and incomplete UTF-8 between chunks.
- `AnsiSpanBuffer` accumulates streaming output and can render the accumulated
  spans to a `LayoutJob`.

## Module Responsibilities

- `model`: public ANSI model types such as `AnsiSpan`, `AnsiStyle`, and
  `AnsiColor`.
- `theme`: egui color/theme mapping, including the xterm palette.
- `sgr`: SGR parameter interpretation and style-state transitions.
- `parser`: `vte::Parser` integration and streaming state.
- `egui_render`: conversion from ANSI spans and ANSI byte streams into
  `LayoutJob`.

## Rendering Policy

`LayoutJob` is the only egui rendering output because ANSI text is usually one
logical string with multiple styles inside it. Keeping it as one widget
preserves wrapping and layout behavior.

Some ANSI semantics are richer than egui's current text format. Curly, dotted,
and dashed underline styles are preserved in `AnsiSpan`, but all underline
variants render as an egui underline stroke.

## Streaming Policy

Streaming input is byte-oriented. This allows callers to feed process output,
PTY output, network logs, or async chunks without pre-splitting valid UTF-8.
`vte` handles partial UTF-8 and escape state between `push_bytes` calls.

`AnsiStreamParser::push_bytes` returns only spans produced by that call.
`AnsiSpanBuffer` is the higher-level helper for callers that want a growing
renderable buffer.

## Performance Policy

The parser uses `vte` instead of regex scanning so it can process incremental
byte streams and avoid reparsing partial escape sequences. The render layer
uses `LayoutJob::append`, letting egui maintain correct UTF-8 byte ranges.

Performance-sensitive paths are covered by Criterion benchmarks:

```sh
cargo bench --bench ansi
```

The benchmark suite covers one-shot parsing, direct `LayoutJob` rendering,
parse-then-render comparison, SGR-dense inputs, truecolor-heavy inputs, long
plain text, chunked streaming parse, and chunked streaming buffer rendering.

## API Policy

The 0.3 API intentionally removes the old compatibility surface:

- Public rendering is centered on `LayoutJob`.
- Semantic parsing is exposed through `AnsiSpan`, `AnsiStyle`, `ansi_to_spans`,
  `ansi_bytes_to_spans`, `AnsiStreamParser`, and `AnsiSpanBuffer`.
- Removed APIs include `AnsiParser`, `ColoredText`, RichText conversion helpers,
  and the old color model helper modules.

## Quality Gates

Before publishing, run:

```sh
cargo fmt -- --check
cargo check --all-targets
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets -- -D warnings
cargo bench --bench ansi --no-run
cargo doc --no-deps
cargo package --allow-dirty
```

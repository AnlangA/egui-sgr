# Contributing

Thanks for improving `egui_sgr`.

## Development Checks

Run the full quality gate before opening a release PR:

```sh
cargo fmt -- --check
cargo check --all-targets
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets --all-features -- -D warnings
cargo bench --bench ansi --no-run
cargo doc --no-deps --all-features
cargo package --allow-dirty
```

## Design Rules

- Keep `LayoutJob` as the primary egui output.
- Keep `AnsiSpan` independent of egui themes so parsed output can be cached.
- Keep stream parsing synchronous and byte-oriented.
- Preserve compatibility APIs unless a major-version release explicitly removes
  them.
- Do not implement terminal screen emulation in this crate; strip unsupported
  control sequences unless a documented render policy is added.

## Tests

Add tests for every new SGR behavior in two layers when possible:

- semantic span parsing, such as `AnsiColor` and `AnsiStyle` state;
- egui rendering, such as `LayoutJob.text`, section ranges, and `TextFormat`.

For public API shape, prefer integration tests under `tests/`.

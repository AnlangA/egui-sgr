use criterion::{Criterion, criterion_group, criterion_main};
use egui_sgr::{
    AnsiSpanBuffer, AnsiStreamParser, EguiAnsiTheme, ansi_to_layout_job, ansi_to_spans,
};
use std::hint::black_box;

const MIXED_SAMPLE: &str = "\
\x1b[1;31merror\x1b[0m: file not found\n\
\x1b[38;5;208mwarning\x1b[0m: slow path used\n\
\x1b[38;2;120;180;255mnote\x1b[0m: truecolor detail\n\
\x1b[48;5;21;38;5;15mwhite on blue\x1b[0m\n\
plain text after reset\n";

const STREAM_CHUNKS: &[&[u8]] = &[
    b"\x1b[32mstream ",
    b"keeps ",
    b"style ",
    b"across ",
    b"chunks\x1b[0m ",
    b"\x1b[38;5;208mthen orange\x1b[0m",
];

fn bench_one_shot_parse(c: &mut Criterion) {
    c.bench_function("ansi_to_spans/mixed", |b| {
        b.iter(|| ansi_to_spans(black_box(MIXED_SAMPLE)));
    });
}

fn bench_layout_job(c: &mut Criterion) {
    let theme = EguiAnsiTheme::default();

    c.bench_function("ansi_to_layout_job/mixed", |b| {
        b.iter(|| ansi_to_layout_job(black_box(MIXED_SAMPLE), black_box(&theme)));
    });
}

fn bench_stream_parser(c: &mut Criterion) {
    c.bench_function("AnsiStreamParser/chunked", |b| {
        b.iter(|| {
            let mut parser = AnsiStreamParser::new();
            let mut spans = Vec::new();

            for chunk in STREAM_CHUNKS {
                spans.extend(parser.push_bytes(black_box(chunk)));
            }
            spans.extend(parser.finish());

            spans
        });
    });
}

fn bench_span_buffer(c: &mut Criterion) {
    let theme = EguiAnsiTheme::default();

    c.bench_function("AnsiSpanBuffer/chunked_to_layout_job", |b| {
        b.iter(|| {
            let mut buffer = AnsiSpanBuffer::new();

            for chunk in STREAM_CHUNKS {
                buffer.push_bytes(black_box(chunk));
            }
            buffer.finish();

            buffer.to_layout_job(black_box(&theme))
        });
    });
}

criterion_group!(
    benches,
    bench_one_shot_parse,
    bench_layout_job,
    bench_stream_parser,
    bench_span_buffer
);
criterion_main!(benches);

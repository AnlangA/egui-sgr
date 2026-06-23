use criterion::{Criterion, criterion_group, criterion_main};
use egui_sgr::{
    AnsiSpanBuffer, AnsiStreamParser, EguiAnsiTheme, ansi_to_layout_job, ansi_to_spans,
    spans_to_layout_job,
};
use std::hint::black_box;

const MIXED_SAMPLE: &str = "\
\x1b[1;31merror\x1b[0m: file not found\n\
\x1b[38;5;208mwarning\x1b[0m: slow path used\n\
\x1b[38;2;120;180;255mnote\x1b[0m: truecolor detail\n\
\x1b[48;5;21;38;5;15mwhite on blue\x1b[0m\n\
plain text after reset\n";

const SGR_DENSE_SAMPLE: &str = "\
\x1b[31ma\x1b[32mb\x1b[33mc\x1b[34md\x1b[35me\x1b[36mf\x1b[0mg\
\x1b[1;31mh\x1b[22;38;5;208mi\x1b[48;5;21;38;5;15mj\x1b[0mk\
\x1b[4;58;5;196ml\x1b[24;59mm\x1b[7mn\x1b[27mo";

const TRUECOLOR_DENSE_SAMPLE: &str = "\
\x1b[38;2;255;105;180mhot\x1b[0m \
\x1b[38:2::120:180:255mcolon\x1b[0m \
\x1b[48;2;30;30;30;38;2;180;220;255msoft\x1b[0m \
\x1b[4:3;58:5:51munderline\x1b[0m";

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

fn bench_sgr_dense_parse(c: &mut Criterion) {
    c.bench_function("ansi_to_spans/sgr_dense", |b| {
        b.iter(|| ansi_to_spans(black_box(SGR_DENSE_SAMPLE)));
    });
}

fn bench_layout_job(c: &mut Criterion) {
    let theme = EguiAnsiTheme::default();

    c.bench_function("ansi_to_layout_job/mixed", |b| {
        b.iter(|| ansi_to_layout_job(black_box(MIXED_SAMPLE), black_box(&theme)));
    });
}

fn bench_parse_then_layout_job(c: &mut Criterion) {
    let theme = EguiAnsiTheme::default();

    c.bench_function("parse_then_spans_to_layout_job/mixed", |b| {
        b.iter(|| {
            let spans = ansi_to_spans(black_box(MIXED_SAMPLE));
            spans_to_layout_job(black_box(&spans), black_box(&theme))
        });
    });
}

fn bench_sgr_dense_layout_job(c: &mut Criterion) {
    let theme = EguiAnsiTheme::default();

    c.bench_function("ansi_to_layout_job/sgr_dense", |b| {
        b.iter(|| ansi_to_layout_job(black_box(SGR_DENSE_SAMPLE), black_box(&theme)));
    });
}

fn bench_truecolor_dense_layout_job(c: &mut Criterion) {
    let theme = EguiAnsiTheme::default();

    c.bench_function("ansi_to_layout_job/truecolor_dense", |b| {
        b.iter(|| ansi_to_layout_job(black_box(TRUECOLOR_DENSE_SAMPLE), black_box(&theme)));
    });
}

fn bench_long_plain_layout_job(c: &mut Criterion) {
    let theme = EguiAnsiTheme::default();
    let long_plain = "plain log line without sgr\n".repeat(256);

    c.bench_function("ansi_to_layout_job/long_plain", |b| {
        b.iter(|| ansi_to_layout_job(black_box(&long_plain), black_box(&theme)));
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
    bench_sgr_dense_parse,
    bench_layout_job,
    bench_parse_then_layout_job,
    bench_sgr_dense_layout_job,
    bench_truecolor_dense_layout_job,
    bench_long_plain_layout_job,
    bench_stream_parser,
    bench_span_buffer
);
criterion_main!(benches);

use blaze_compiler::{lexer::lex, parser::parse};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parser_small(c: &mut Criterion) {
    let source = "fn add(a: i32, b: i32) -> i32 { a + b }";

    c.bench_function("parser_small", |b| {
        b.iter(|| {
            let tokens = lex(black_box(source)).unwrap();
            parse(black_box(tokens))
        })
    });
}

fn bench_parser_medium(c: &mut Criterion) {
    let source = include_str!("../examples/structs_and_enums.blz");

    c.bench_function("parser_medium", |b| {
        b.iter(|| {
            let tokens = lex(black_box(source)).unwrap();
            parse(black_box(tokens))
        })
    });
}

fn bench_parser_large(c: &mut Criterion) {
    let mut source = String::new();
    for i in 0..100 {
        source.push_str(&format!(
            "struct Point{} {{ x: f64, y: f64 }}\nimpl Point{} {{ fn new(x: f64, y: f64) -> Point{} {{ Point{} {{ x, y }} }} }}\n",
            i, i, i, i
        ));
    }

    c.bench_function("parser_large", |b| {
        b.iter(|| {
            let tokens = lex(black_box(&source)).unwrap();
            parse(black_box(tokens))
        })
    });
}

fn bench_parser_complex_expr(c: &mut Criterion) {
    let source = "if x > 0 { x * 2 } else { x / 2 } + match y { 1 => 10, 2 => 20, _ => 0 }";

    c.bench_function("parser_complex_expr", |b| {
        b.iter(|| {
            let tokens = lex(black_box(source)).unwrap();
            parse(black_box(tokens))
        })
    });
}

criterion_group!(
    benches,
    bench_parser_small,
    bench_parser_medium,
    bench_parser_large,
    bench_parser_complex_expr
);
criterion_main!(benches);

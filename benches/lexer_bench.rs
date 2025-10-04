use blaze_compiler::lexer::lex;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_lexer_small(c: &mut Criterion) {
    let source = "let x = 42; let y = 3.14; let s = \"hello world\";";

    c.bench_function("lexer_small", |b| b.iter(|| lex(black_box(source))));
}

fn bench_lexer_medium(c: &mut Criterion) {
    let source = include_str!("../examples/hello_world.blz");

    c.bench_function("lexer_medium", |b| b.iter(|| lex(black_box(source))));
}

fn bench_lexer_large(c: &mut Criterion) {
    let mut source = String::new();
    for i in 0..1000 {
        source.push_str(&format!("fn func_{}() {{ let x = {}; }}\n", i, i));
    }

    c.bench_function("lexer_large", |b| b.iter(|| lex(black_box(&source))));
}

fn bench_lexer_unicode(c: &mut Criterion) {
    let source = "let 世界 = \"hello 世界\"; let emoji = \"🚀🔥💯\";";

    c.bench_function("lexer_unicode", |b| b.iter(|| lex(black_box(source))));
}

criterion_group!(
    benches,
    bench_lexer_small,
    bench_lexer_medium,
    bench_lexer_large,
    bench_lexer_unicode
);
criterion_main!(benches);

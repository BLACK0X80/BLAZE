use criterion::{black_box, criterion_group, criterion_main, Criterion};
use blaze_compiler::lexer::lex;
use blaze_compiler::parser::parse;
use blaze_compiler::ir::generate;

fn codegen_benchmark(c: &mut Criterion) {
    let source = r#"
        fn factorial(n: i32) -> i32 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        
        fn main() {
            let result = factorial(10);
            println("{}", result);
        }
    "#;
    
    c.bench_function("codegen_factorial", |b| {
        b.iter(|| {
            let tokens = lex(black_box(source)).unwrap();
            let ast = parse(tokens).unwrap();
            generate(&ast).unwrap()
        })
    });
}

criterion_group!(benches, codegen_benchmark);
criterion_main!(benches);

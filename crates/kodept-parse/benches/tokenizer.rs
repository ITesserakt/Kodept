use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use kodept_parse::lexer::{PegLexer, PestLexer};
use kodept_parse::tokenizer::{EagerTokenizer, LazyTokenizer, ParallelTokenizer, Tok, TokCtor};

const FILENAME: &str = "benches/benchmarking_file1.kd";

fn get_contents_with_factor(filename: &str, factor: usize) -> String {
    let contents = std::fs::read_to_string(filename).unwrap();
    contents.repeat(factor)
}

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");
    for factor in [1, 10, 100, 1000] {
        let contents = get_contents_with_factor(FILENAME, factor);
        group.throughput(Throughput::Bytes(contents.as_bytes().len() as u64));

        group.bench_with_input(BenchmarkId::new("peg", factor), &contents, |b, i| {
            b.iter(|| EagerTokenizer::new(i, PegLexer::<false>::new()).into_vec())
        });
        group.bench_with_input(BenchmarkId::new("pest", factor), &contents, |b, i| {
            b.iter(|| EagerTokenizer::new(i, PestLexer::new()).into_vec())
        });
        group.bench_with_input(BenchmarkId::new("lazy-pest", factor), &contents, |b, i| {
            b.iter(|| LazyTokenizer::new(i, PestLexer::new()).into_vec())
        });
        group.bench_with_input(
            BenchmarkId::new("parallel-peg", factor),
            &contents,
            |b, i| b.iter(|| ParallelTokenizer::new(i, PegLexer::<false>::new()).into_vec()),
        );
        group.bench_with_input(
            BenchmarkId::new("parallel-pest", factor),
            &contents,
            |b, i| b.iter(|| ParallelTokenizer::new(i, PestLexer::new()).into_vec()),
        );
    }
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

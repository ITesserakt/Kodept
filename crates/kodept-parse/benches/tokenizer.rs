use criterion::measurement::Measurement;
use criterion::{
    criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion, Throughput,
};
use kodept_parse::lexer::{NomLexer, PegLexer, PestLexer};
use kodept_parse::tokenizer::Tokenizer;

const FILENAMES: [(&str, &str); 6] = [
    ("benches/benchmarking_file1.kd", "large"),
    ("benches/benchmarking_file2.kd", "simple1"),
    ("benches/benchmarking_file3.kd", "simple2"),
    ("benches/benchmarking_file4.kd", "medium"),
    ("benches/benchmarking_file5.kd", "half-large"),
    ("benches/benchmarking_file6.kd", "well-fed"),
];

fn make_bench_impl<M, F, T: Tokenizer<F>>(
    group: &mut BenchmarkGroup<M>,
    name: &'static str,
    description: &'static str,
    input: &str,
    lexer: F,
) where
    M: Measurement,
{
    group.bench_with_input(BenchmarkId::new(name, description), input, |b, i| {
        b.iter(|| T::new(i, lexer).into_vec())
    });
}

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");
    for (name, description) in FILENAMES {
        let contents = std::fs::read_to_string(name).unwrap();
        group.throughput(Throughput::Bytes(contents.as_bytes().len() as u64));

        make_bench_impl(&mut group, "nom", description, &contents, NomLexer::new());
        make_bench_impl(&mut group, "peg", description, &contents, PegLexer::new());
        make_bench_impl(&mut group, "pest", description, &contents, PestLexer::new());
    }
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

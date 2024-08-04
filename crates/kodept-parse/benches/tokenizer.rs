use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main, Throughput};

use kodept_parse::lexer::{NomLexer, PegLexer, PestLexer};
use kodept_parse::tokenizer::GenericLazyTokenizer;

const FILENAMES: [(&str, &str); 6] = [
    ("benches/benchmarking_file1.kd", "large"),
    ("benches/benchmarking_file2.kd", "simple1"),
    ("benches/benchmarking_file3.kd", "simple2"),
    ("benches/benchmarking_file4.kd", "medium"),
    ("benches/benchmarking_file5.kd", "half-large"),
    ("benches/benchmarking_file6.kd", "well-fed"),
];

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");
    for (name, description) in FILENAMES {
        let contents = std::fs::read_to_string(name).unwrap();
        group.throughput(Throughput::Bytes(contents.as_bytes().len() as u64));
        group.bench_with_input(
            BenchmarkId::new("default", description),
            &contents,
            |b, i| b.iter(|| GenericLazyTokenizer::new(i, NomLexer::new()).into_vec()),
        );
        group.bench_with_input(BenchmarkId::new("pest", description), &contents, |b, i| {
            b.iter(|| GenericLazyTokenizer::new(i, PestLexer::new()).into_vec())
        });
        group.bench_with_input(BenchmarkId::new("peg", description), &contents, |b, i| {
            b.iter(|| GenericLazyTokenizer::new(i, PegLexer::new()).into_vec())
        });
    }
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

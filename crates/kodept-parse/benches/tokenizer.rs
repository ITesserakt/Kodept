use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use kodept_parse::lexer::{PegLexer, PestLexer};
use kodept_parse::tokenizer::{EagerTokenizer, ParallelTokenizer, Tokenizer};

const FILENAMES: [(&str, &str); 6] = [
    ("benches/benchmarking_file1.kd", "30*128"),
    ("benches/benchmarking_file2.kd", "30*256"),
    ("benches/benchmarking_file3.kd", "30*8"),
    ("benches/benchmarking_file4.kd", "30*16"),
    ("benches/benchmarking_file5.kd", "30*32"),
    ("benches/benchmarking_file6.kd", "30*64"),
];

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");
    for (name, description) in FILENAMES {
        let contents = std::fs::read_to_string(name).unwrap();
        group.throughput(Throughput::Bytes(contents.as_bytes().len() as u64));

        // group.bench_with_input(BenchmarkId::new("nom", description), &contents, |b, i| {
        //     b.iter(move || {
        //         LazyTokenizer::new(i, NomLexer::new()).into_vec();
        //     })
        // });
        group.bench_with_input(BenchmarkId::new("peg", description), &contents, |b, i| {
            b.iter(|| EagerTokenizer::new(i, PegLexer::<false>::new()).into_vec())
        });
        group.bench_with_input(BenchmarkId::new("pest", description), &contents, |b, i| {
            b.iter(|| EagerTokenizer::new(i, PestLexer::new()).into_vec())
        });
        group.bench_with_input(
            BenchmarkId::new("parallel-peg", description),
            &contents,
            |b, i| b.iter(|| ParallelTokenizer::new(i, PegLexer::<false>::new()).into_vec()),
        );
        group.bench_with_input(
            BenchmarkId::new("parallel-pest", description),
            &contents,
            |b, i| b.iter(|| ParallelTokenizer::new(i, PestLexer::new()).into_vec()),
        );
    }
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

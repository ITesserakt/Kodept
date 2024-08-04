use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main, Throughput};

use kodept_parse::parser::{NomParser, PegParser};
use kodept_parse::token_match::TokenMatch;
use kodept_parse::token_stream::TokenStream;
use kodept_parse::tokenizer::LazyTokenizer;
use kodept_parse::common::RLTProducer;

fn get_tokens_from_contents(contents: &str) -> Vec<TokenMatch> {
    let tokenizer = LazyTokenizer::new(contents);
    let tokens = tokenizer.into_vec();
    tokens
}

const FILENAMES: [(&str, &str); 6] = [
    ("benches/benchmarking_file1.kd", "large"),
    ("benches/benchmarking_file2.kd", "simple1"),
    ("benches/benchmarking_file3.kd", "simple2"),
    ("benches/benchmarking_file4.kd", "medium"),
    ("benches/benchmarking_file5.kd", "half-large"),
    ("benches/benchmarking_file6.kd", "well-fed"),
];

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    for (name, description) in FILENAMES {
        let contents = std::fs::read_to_string(name).unwrap();
        let tokens = get_tokens_from_contents(&contents);
        let tokens = TokenStream::new(&tokens);
        group.throughput(Throughput::Bytes(contents.as_bytes().len() as u64));

        group.bench_with_input(BenchmarkId::new("default", description), &tokens, |b, i| {
            b.iter(|| NomParser::new().parse_rlt(*i).expect("Success"))
        });
        group.bench_with_input(BenchmarkId::new("peg", description), &tokens, |b, i| {
            b.iter(|| PegParser::new().parse_rlt(*i).expect("Success"))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

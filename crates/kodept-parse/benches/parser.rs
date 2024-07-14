use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main, Throughput};
use nom_supreme::final_parser::final_parser;

use kodept_parse::{parse_from_top, ParseError};
use kodept_parse::parser::file::grammar;
use kodept_parse::token_match::TokenMatch;
use kodept_parse::token_stream::TokenStream;
use kodept_parse::tokenizer::Tokenizer;

fn get_tokens_from_contents(contents: &str) -> Vec<TokenMatch> {
    let tokenizer = Tokenizer::new(contents);
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
            b.iter(|| {
                let res: Result<_, ParseError> = final_parser(grammar)(*i);
                res.expect("Success")
            })
        });
        group.bench_with_input(BenchmarkId::new("peg", description), &tokens, |b, i| {
            b.iter(|| parse_from_top(*i).expect("Success"))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

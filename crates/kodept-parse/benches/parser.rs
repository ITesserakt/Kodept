use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use kodept_parse::common::RLTProducer;
use kodept_parse::lexer::PegLexer;
use kodept_parse::parser::{NomParser, PegParser};
use kodept_parse::token_match::PackedTokenMatch;
use kodept_parse::token_stream::PackedTokenStream;
use kodept_parse::tokenizer::{LazyTokenizer, Tok, TokCtor};

const FILENAME: &str = "benches/benchmarking_file1.kd";

fn get_contents_with_factor(filename: &str, factor: usize) -> String {
    let contents = std::fs::read_to_string(filename).unwrap();
    contents.repeat(factor)
}

fn get_tokens_from_contents(contents: &str) -> Vec<PackedTokenMatch> {
    let tokenizer = LazyTokenizer::new(contents, PegLexer::<false>::new());
    tokenizer.into_vec()
}

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    for factor in (5..=10).map(|it| 2usize.pow(it)) {
        let contents = get_contents_with_factor(FILENAME, factor);
        let tokens = get_tokens_from_contents(&contents);
        let tokens = PackedTokenStream::new(&tokens);
        group.throughput(Throughput::Bytes(contents.as_bytes().len() as u64));

        group.bench_with_input(BenchmarkId::new("nom", factor), &tokens, |b, i| {
            b.iter(|| NomParser::new().parse_stream(i).expect("Success"))
        });
        group.bench_with_input(BenchmarkId::new("peg", factor), &tokens, |b, i| {
            b.iter(|| PegParser::<false>::new().parse_stream(i).expect("Success"))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use kodept_ast::graph::SyntaxTree;
use kodept_ast::interning::InterningCodeHolder;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::rlt::RLT;
use kodept_core::structure::span::CodeHolder;
use kodept_parse::common::{EagerTokensProducer, RLTProducer};
use kodept_parse::lexer::PestLexer;
use kodept_parse::parser::PegParser;
use kodept_parse::token_stream::PackedTokenStream;
use std::sync::LazyLock;

const FILE_CONTENTS: &str = include_str!("benchmarking_file1.kd");

static PARSED_FILE: LazyLock<RLT> = LazyLock::new(|| {
    let lexer = PestLexer::new();
    let tokens = lexer.parse_string(FILE_CONTENTS).unwrap();
    let stream = PackedTokenStream::new(&tokens);
    let parser = PegParser::<{ kodept_parse::TRACING_OPTION }>::new();

    parser.parse_stream(&stream).unwrap()
});

#[derive(Debug, Copy, Clone)]
struct InlineCodeHolder(&'static str);
impl CodeHolder for InlineCodeHolder {
    type Str = &'static str;

    fn get_chunk(self, at: CodePoint) -> Self::Str {
        &self.0[at.as_range()]
    }
}

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_building");
    let rlt = PARSED_FILE.clone();
    let sources = InlineCodeHolder(FILE_CONTENTS);
    group.throughput(Throughput::Bytes(FILE_CONTENTS.as_bytes().len() as u64));

    group.bench_with_input("main", &(rlt, sources), |b, (rlt, sources)| {
        b.iter_with_large_drop(|| {
            let sources = InterningCodeHolder::new(*sources);
            SyntaxTree::<()>::recursively_build(rlt, sources);
        })
    });
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

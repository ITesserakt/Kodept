use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use kodept_ast::syntax_tree::prelude::AST;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use kodept_parse::common::{EagerTokensProducer, RLTProducer};
use kodept_parse::lexer::PestLexer;
use kodept_parse::parser::PegParser;
use kodept_parse::token_stream::PackedTokenStream;
use kodept_rlt::prelude::RLT;
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

    #[cfg(all(not(feature = "interning"), not(feature = "parallel")))]
    group.bench_with_input(
        "no interning, no parallelization",
        &(rlt, sources),
        |b, (rlt, sources)| {
            b.iter_with_large_drop(move || {
                let sources = kodept::read_code_source::CloningCodeHolder::new(*sources);
                AST::recursively_build::<FileDecl>(rlt.clone(), sources);
            })
        },
    );

    #[cfg(all(feature = "interning", not(feature = "parallel")))]
    group.bench_with_input(
        "interning, no parallelization",
        &(rlt, sources),
        |b, (rlt, sources)| {
            b.iter_with_large_drop(move || {
                let sources = kodept_interning::InterningCodeHolder::new(*sources);
                AST::recursively_build::<FileDecl>(rlt.clone(), sources);
            })
        },
    );

    #[cfg(all(not(feature = "interning"), feature = "parallel"))]
    group.bench_with_input(
        "no interning, parallelization",
        &(rlt, sources),
        |b, (rlt, sources)| {
            b.iter_with_large_drop(move || {
                let sources = kodept::read_code_source::CloningCodeHolder::new(*sources);
                AST::recursively_build::<FileDecl>(rlt.clone(), sources);
            })
        },
    );

    #[cfg(all(feature = "interning", feature = "parallel"))]
    group.bench_with_input(
        "interning, parallelization",
        &(rlt, sources),
        |b, (rlt, sources)| {
            b.iter_with_large_drop(move || {
                let sources = kodept_interning::InterningCodeHolder::new(*sources);
                AST::recursively_build::<FileDecl>(rlt.clone(), sources);
            })
        },
    );
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use kodept_ast::graph::SyntaxTree;
use kodept_ast::interning::SharedStr;
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
    type Str = SharedStr;

    fn get_chunk(self, at: CodePoint) -> Self::Str {
        SharedStr::new(&self.0[at.as_range()])
    }
}

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_building");
    let rlt = PARSED_FILE.clone();
    let sources = InlineCodeHolder(FILE_CONTENTS);
    group.throughput(Throughput::Bytes(FILE_CONTENTS.as_bytes().len() as u64));

    #[cfg(not(feature = "parallel"))]
    group.bench_with_input("no parallelism", &(rlt, sources), |b, (rlt, sources)| {
        b.iter_with_large_drop(move || {
            SyntaxTree::<()>::recursively_build(rlt, *sources);
        })
    });

    #[cfg(feature = "parallel")]
    {
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .use_current_thread()
            .build_global()
            .unwrap();

        for parallelism in 1..16 {
            let pool = build_thread_pool(parallelism);

            group.bench_with_input(
                criterion::BenchmarkId::new("parallelism", parallelism),
                &(&rlt, sources),
                |b, (rlt, sources)| {
                    pool.install(|| {
                        b.iter(|| SyntaxTree::<()>::recursively_build(rlt, *sources))
                    })
                },
            );
        }
    }
}

#[cfg(feature = "parallel")]
fn build_thread_pool(size: usize) -> rayon::ThreadPool {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(size)
        .build()
        .unwrap();
    pool
}

criterion_group!(benches, bench_impls);
criterion_main!(benches);

use std::borrow::Cow;
use criterion::{criterion_group, Criterion, Throughput};
use kodept_ast::graph::SyntaxTree;
use kodept_core::code_point::CodePoint;
use kodept_rlt::prelude::{File, RLT};
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

fn parsed_file(modules: usize) -> RLT {
    let module = PARSED_FILE.0 .0.first().unwrap();
    let modules = (0..modules).map(|_| module.clone()).collect::<Box<_>>();
    RLT(File(modules))
}

#[derive(Debug, Copy, Clone)]
struct InlineCodeHolder(&'static str);
impl CodeHolder for InlineCodeHolder {
    type Str = Cow<'static, str>;
    
    fn get_chunk(self, at: CodePoint) -> Self::Str {
        Cow::Borrowed(&self.0[at.as_range()])
    }
}

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_building");
    let rlt = parsed_file(200);
    let sources = InlineCodeHolder(FILE_CONTENTS);
    group.throughput(Throughput::Elements(200));

    #[cfg(not(feature = "parallel"))]
    group.bench_with_input("no parallelism", &(rlt, sources), |b, (rlt, sources)| {
        b.iter_with_large_drop(move || {
            SyntaxTree::<()>::recursively_build(rlt, *sources);
        })
    });

    #[cfg(feature = "parallel")]
    {
        for parallelism in 1..16 {
            let pool = build_thread_pool(parallelism);

            group.bench_function(
                criterion::BenchmarkId::new("parallelism", parallelism),
                |b| pool.install(|| b.iter(|| SyntaxTree::<()>::recursively_build(&rlt, sources))),
            );
        }
    }
}

fn bench_complexity(c: &mut Criterion) {
    #[cfg(feature = "parallel")]
    let pool = build_thread_pool(9);
    let mut group = c.benchmark_group("ast_building");
    for size in [1, 2, 5, 10, 50, 200, 400, 700] {
        let rlt = parsed_file(size);
        let sources = InlineCodeHolder(FILE_CONTENTS);

        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(criterion::BenchmarkId::new("complexity", size), |b| {
            #[cfg(feature = "parallel")]
            return pool.install(|| b.iter(|| SyntaxTree::<()>::recursively_build(&rlt, sources)));
            #[cfg(not(feature = "parallel"))]
            return b.iter(|| SyntaxTree::<()>::recursively_build(&rlt, sources));
        });
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

criterion_group!(benches, bench_impls, bench_complexity);

fn main() {
    #[cfg(feature = "parallel")]
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .use_current_thread()
        .build_global()
        .unwrap();

    benches();

    Criterion::default().configure_from_args().final_summary();
}

use criterion::{criterion_group, Criterion, Throughput};
use kodept_ast::graph::SyntaxTree;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use kodept_parse::common::{EagerTokensProducer, RLTProducer};
use kodept_parse::lexer::PestLexer;
use kodept_parse::parser::PegParser;
use kodept_parse::token_stream::PackedTokenStream;
use kodept_rlt::prelude as rlt;
use kodept_rlt::prelude::RLT;
use std::borrow::Cow;
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
    RLT(rlt::File(modules))
}

#[derive(Debug, Copy, Clone)]
struct InlineCodeHolder(&'static str);
impl CodeHolder for InlineCodeHolder {
    type Str = &'static str;

    fn get_chunk(self, at: CodePoint) -> Self::Str {
        &self.0[at.as_range()]
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

fn bench_complexity(c: &mut Criterion) {
    #[cfg(feature = "parallel")]
    let pool = build_thread_pool(9);
    let mut group = c.benchmark_group("ast_building");
    for size in [1, 2, 5, 10, 50, 200, 400, 700] {
        let rlt = parsed_file(size);
        const ID: &'static str = if cfg!(feature = "parallel") {
            "complexity/p"
        } else {
            "complexity/np"
        };

        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(criterion::BenchmarkId::new(ID, size), |b| {
            #[cfg(feature = "parallel")]
            return pool.install(|| {
                b.iter(|| {
                    SyntaxTree::recursively_build(
                        &rlt,
                        InlineCodeHolder(FILE_CONTENTS).map(|it| Cow::Borrowed(it)),
                    )
                })
            });
            #[cfg(not(feature = "parallel"))]
            return b.iter(|| {
                SyntaxTree::recursively_build(
                    &rlt,
                    InlineCodeHolder(FILE_CONTENTS).map(Cow::Borrowed),
                )
            });
        });
    }
}

#[cfg(feature = "parallel")]
fn parallel_bench<M, C>(id: &str, group: &mut criterion::BenchmarkGroup<M>, sources: C)
where
    M: criterion::measurement::Measurement<Value: Send> + Sync,
    C: CodeHolder<Str = Cow<'static, str>>,
{
    for parallelism in 1..11 {
        let pool = build_thread_pool(parallelism);
        let rlt = &*PARSED_FILE;

        group.bench_function(criterion::BenchmarkId::new(id, parallelism), |b| {
            pool.install(|| b.iter(|| SyntaxTree::recursively_build(rlt, sources)))
        });
    }
}

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_building");
    let sources = InlineCodeHolder(FILE_CONTENTS);
    group.throughput(Throughput::Bytes(FILE_CONTENTS.len() as u64));

    #[cfg(all(not(feature = "interning"), not(feature = "parallel")))]
    group.bench_function("no interning, no parallelization", |b| {
        let sources = sources.map(Cow::from);
        let rlt = &*PARSED_FILE;

        b.iter(|| SyntaxTree::recursively_build(rlt, sources))
    });

    #[cfg(all(feature = "interning", not(feature = "parallel")))]
    group.bench_function("interning, no parallelization", |b| {
        let sources = kodept_interning::InterningCodeHolder::new(sources).map(|it| Cow::from(it.0));
        b.iter_batched(
            || PARSED_FILE.clone(),
            |rlt| AST::recursively_build::<FileDecl>(rlt.clone(), sources),
            BatchSize::SmallInput,
        )
    });

    #[cfg(all(not(feature = "interning"), feature = "parallel"))]
    parallel_bench(
        "no interning, parallelization",
        &mut group,
        sources.map(Cow::from),
    );

    #[cfg(all(feature = "interning", feature = "parallel"))]
    parallel_bench(
        "interning, parallelization",
        &mut group,
        kodept_interning::InterningCodeHolder::new(sources).map(|it| Cow::from(it.0)),
    );
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

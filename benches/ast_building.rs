use kodept_core::code_point::CodePoint;
use criterion::{criterion_group, BatchSize, Criterion, Throughput};
use kodept_ast::syntax_tree::prelude::{SourceCode, AST};
use kodept_ast_nodes::file::FileDecl;
use kodept_core::structure::span::CodeHolder;
use kodept_parse::common::{EagerTokensProducer, RLTProducer};
use kodept_parse::lexer::ASCIILexer;
use kodept_parse::parser::PegParser;
use kodept_parse::token_stream::PackedTokenStream;
use kodept_rlt::prelude as rlt;
use kodept_rlt::prelude::RLT;
use std::borrow::Cow;
use std::sync::LazyLock;
use kodept_core::file_name::{FileDescriptor, FileId, FileName};

const FILE_CONTENTS: &str = include_str!("benchmarking_file1.kd");
const MODULES_COUNT: u64 = 10;

static PARSED_FILE: LazyLock<RLT> = LazyLock::new(|| {
    let lexer = ASCIILexer::new();
    let tokens = lexer.parse_string(FILE_CONTENTS).unwrap();
    let stream = PackedTokenStream::new(&tokens);
    let parser = PegParser::<{ kodept_parse::TRACING_OPTION }>::new();

    parser.parse_stream(&stream).unwrap()
});

fn parsed_file(modules: u64) -> RLT {
    let module = PARSED_FILE.0 .0.first().unwrap();
    let modules = (0..modules).map(|_| module.clone()).collect::<Box<_>>();
    RLT(rlt::File(modules))
}

fn wrap_source_code<S: CodeHolder<Str = Cow<'static, str>>>(holder: S) -> SourceCode<S> {
    SourceCode::new(holder, FileDescriptor::new(FileName::Anon, FileId::generate()))
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

        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(criterion::BenchmarkId::new("complexity", size), |b| {
            #[cfg(feature = "parallel")]
            return pool.install(|| {
                b.iter_batched(
                    || rlt.clone(),
                    |rlt| {
                        let code_holder = InlineCodeHolder(FILE_CONTENTS).map(|it| Cow::Borrowed(it));
                        AST::recursively_build::<FileDecl>(
                            rlt,
                            wrap_source_code(code_holder),
                        )
                    },
                    BatchSize::SmallInput,
                )
            });
            #[cfg(not(feature = "parallel"))]
            return b.iter_batched(
                || rlt.clone(),
                |rlt| {
                    let code_holder = InlineCodeHolder(FILE_CONTENTS).map(Cow::Borrowed);
                    AST::recursively_build::<FileDecl>(
                        rlt,
                        wrap_source_code(code_holder),
                    )
                },
                BatchSize::SmallInput,
            );
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

        group.bench_function(criterion::BenchmarkId::new(id, parallelism), |b| {
            pool.install(|| {
                b.iter_batched(
                    || parsed_file(MODULES_COUNT),
                    |rlt| AST::recursively_build::<FileDecl>(rlt, wrap_source_code(sources)),
                    BatchSize::SmallInput,
                )
            })
        });
    }
}

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_building");
    let sources = InlineCodeHolder(FILE_CONTENTS);
    group.throughput(Throughput::Elements(MODULES_COUNT));

    #[cfg(all(not(feature = "interning"), not(feature = "parallel")))]
    group.bench_function("no interning, no parallelization", |b| {
        let sources = sources.map(Cow::from);
        b.iter_batched(
            || parsed_file(MODULES_COUNT),
            |rlt| AST::recursively_build::<FileDecl>(rlt.clone(), wrap_source_code(sources)),
            BatchSize::SmallInput,
        )
    });

    #[cfg(all(feature = "interning", not(feature = "parallel")))]
    group.bench_function("interning, no parallelization", |b| {
        let sources = kodept_interning::InterningCodeHolder::new(sources).map(|it| Cow::from(it.0));
        b.iter_batched(
            || parsed_file(MODULES_COUNT),
            |rlt| AST::recursively_build::<FileDecl>(rlt.clone(), wrap_source_code(sources)),
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

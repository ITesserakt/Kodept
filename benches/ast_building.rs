use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use kodept_ast::syntax_tree::prelude::AST;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use kodept_parse::common::{EagerTokensProducer, RLTProducer};
use kodept_parse::lexer::PestLexer;
use kodept_parse::parser::PegParser;
use kodept_parse::token_stream::PackedTokenStream;
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

#[derive(Debug, Copy, Clone)]
struct InlineCodeHolder(&'static str);
impl CodeHolder for InlineCodeHolder {
    type Str = &'static str;

    fn get_chunk(self, at: CodePoint) -> Self::Str {
        &self.0[at.as_range()]
    }
}

#[cfg(feature = "parallel")]
fn parallel_bench<M, C>(id: &str, group: &mut criterion::BenchmarkGroup<M>, sources: C)
where
    M: criterion::measurement::Measurement<Value: Send> + Sync,
    C: CodeHolder<Str = Cow<'static, str>>,
{
    rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .use_current_thread()
        .build_global()
        .unwrap();

    for parallelism in 1..15 {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(parallelism)
            .build()
            .unwrap();

        group.bench_function(criterion::BenchmarkId::new(id, parallelism), |b| {
            pool.install(|| {
                b.iter_batched(
                    || PARSED_FILE.clone(),
                    |rlt| AST::recursively_build::<FileDecl>(rlt, sources),
                    BatchSize::SmallInput,
                )
            })
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
        b.iter_batched(
            || PARSED_FILE.clone(),
            |rlt| AST::recursively_build::<FileDecl>(rlt.clone(), sources),
            BatchSize::SmallInput,
        )
    });

    #[cfg(all(feature = "interning", not(feature = "parallel")))]
    group.bench_function("no interning, no parallelization", |b| {
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

criterion_group!(benches, bench_impls);
criterion_main!(benches);

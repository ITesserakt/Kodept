extern crate core;

use bevy_ecs::prelude::{Mut, World};
use criterion::measurement::Measurement;
use criterion::{criterion_group, BatchSize, Bencher, Criterion, Throughput};
use kodept_ast::prelude::NodeId;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::experimental::GenericSpawnContext;
use kodept_ast_nodes::module::Module;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use kodept_rlt::prelude::{File, RLT};
use std::borrow::Cow;
use std::sync::LazyLock;

const FILE_CONTENTS: &str = include_str!("benchmarking_file1.kd");
const SERIALIZED_RLT_CONTENTS: &str = include_str!("benchmarking_file1.rlt.json");
const MODULES_COUNT: u64 = 10;

static PARSED_FILE: LazyLock<RLT> =
    LazyLock::new(|| serde_json::from_str(SERIALIZED_RLT_CONTENTS).unwrap());

fn parsed_file(modules: u64) -> SyntaxResolver {
    let module = PARSED_FILE.0 .0.first().unwrap();
    let modules = (0..modules).map(|_| module.clone()).collect::<Box<_>>();
    SyntaxResolver::build(RLT(File(modules)))
}

fn bench_fns<M: Measurement>(size: u64) -> impl FnMut(&mut Bencher<M>) {
    move |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                world.insert_resource(parsed_file(size));
                (
                    world,
                    InlineCodeHolder(FILE_CONTENTS).map(|s| Cow::Borrowed(s)),
                )
            },
            |(mut world, code)| {
                world.resource_scope(|w, syntax: Mut<SyntaxResolver>| {
                    for module in &syntax.root().0 .0 {
                        let _: NodeId<Module> =
                            GenericSpawnContext::top_level(module, w.commands(), code).unwrap();
                    }
                });
            },
            BatchSize::SmallInput,
        )
    }
}

#[derive(Debug, Copy, Clone)]
struct InlineCodeHolder(&'static str);
impl CodeHolder for InlineCodeHolder {
    type Str = &'static str;

    #[inline(always)]
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
    let mut group = c.benchmark_group("ast_building");
    for size in [1, 2, 5, 10, 50, 200, 400, 700] {
        group.throughput(Throughput::Elements(size));
        #[cfg(feature = "parallel")]
        fn bencher(size: u64) -> impl FnMut(&mut Bencher) {
            static POOL: LazyLock<rayon::ThreadPool> = LazyLock::new(|| build_thread_pool(9));
            move |b| POOL.install(|| bench_fns(size)(b))
        }
        #[cfg(not(feature = "parallel"))]
        fn bencher(size: u64) -> impl FnMut(&mut Bencher) {
            bench_fns(size)
        }

        group.bench_function(
            criterion::BenchmarkId::new("complexity", size),
            bencher(size),
        );
    }
}

#[cfg(feature = "parallel")]
fn parallel_bench<M>(id: &str, group: &mut criterion::BenchmarkGroup<M>)
where
    M: Measurement<Value: Send> + Sync,
{
    for parallelism in 1..11 {
        let pool = build_thread_pool(parallelism);

        group.bench_function(criterion::BenchmarkId::new(id, parallelism), |b| {
            pool.install(|| bench_fns(MODULES_COUNT)(b))
        });
    }
}

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_building");
    group.throughput(Throughput::Elements(MODULES_COUNT));

    #[cfg(not(feature = "parallel"))]
    group.bench_function("no interning, no parallelization", bench_fns(MODULES_COUNT));

    #[cfg(feature = "parallel")]
    parallel_bench("no interning, parallelization", &mut group);
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

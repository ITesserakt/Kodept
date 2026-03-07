use criterion::measurement::Measurement;
use criterion::{BatchSize, Bencher, Criterion, Throughput, criterion_group};
use kodept_ast::experimental::FromSyntax;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::experimental::NodeSpawner;
use kodept_ast_nodes::Module;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::span::CodeHolder;
use kodept_ecs::world::World;
use kodept_rlt::prelude::{File, RLT};
use std::borrow::Cow;
use std::sync::LazyLock;

const FILE_CONTENTS: &str = include_str!("benchmarking_file1.kd");
const SERIALIZED_RLT_CONTENTS: &str = include_str!("benchmarking_file1.rlt.json");
const MODULES_COUNT: u64 = 10;

static PARSED_FILE: LazyLock<RLT> =
    LazyLock::new(|| serde_json::from_str(SERIALIZED_RLT_CONTENTS).unwrap());

fn parsed_file(modules: u64) -> SyntaxResolver {
    let module = PARSED_FILE.0.0.first().unwrap();
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
                let syntax = world.remove_resource::<SyntaxResolver>().unwrap();

                for module in &syntax.root().0.0 {
                    Module::from_syntax(module, NodeSpawner::new(&mut world), code).unwrap();
                }
                world.flush();
            },
            BatchSize::LargeInput,
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

fn bench_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_building");
    for size in [1, 2, 5, 10, 50, 200, 400, 700] {
        group.throughput(Throughput::Elements(size));
        fn bencher(size: u64) -> impl FnMut(&mut Bencher) {
            bench_fns(size)
        }

        group.bench_function(
            criterion::BenchmarkId::new("complexity", size),
            bencher(size),
        );
    }
}

fn bench_impls(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_building");
    group.throughput(Throughput::Elements(MODULES_COUNT));

    group.bench_function("no interning, no parallelization", bench_fns(MODULES_COUNT));
}

criterion_group!(benches, bench_impls, bench_complexity);

fn main() {
    benches();

    Criterion::default().configure_from_args().final_summary();
}

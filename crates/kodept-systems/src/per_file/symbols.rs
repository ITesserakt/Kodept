use bevy_ecs::prelude::ChildOf;
use bevy_ecs::system::Query;
use kodept_ast::Str;
use kodept_ast::prelude::NodeId;
use kodept_ast::properties::Node;
use kodept_ast_nodes::{Path, Unresolved, Value};
use kodept_frontend::define_phase;
use kodept_frontend::engine::PhaseEngine;
use std::iter::repeat;

define_phase! {
    pub phase ReferenceResolutionPhase[ReferenceResolutionPhaseLabel];

    fn build(self, engine: &mut PhaseEngine<Self>) {
        engine.add_systems(resolve_values);
    }
}

struct Symbol<'a> {
    context: &'a Path,
    ident: &'a Str,
}

fn collect_all_symbols(value: &Unresolved) -> impl Iterator<Item = Symbol<'_>> {
    enum Helper<'a, T> {
        Once(std::iter::Once<T>),
        Many(
            std::iter::FlatMap<
                std::slice::Iter<'a, Unresolved>,
                Box<dyn Iterator<Item = T> + 'a>,
                fn(&'a Unresolved) -> Box<dyn Iterator<Item = T> + 'a>,
            >,
        ),
    }

    impl<'a, T> Iterator for Helper<'a, T> {
        type Item = T;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Helper::Once(x) => x.next(),
                Helper::Many(x) => x.next(),
            }
        }
    }

    match value {
        Unresolved::Named { context, ident } => {
            Helper::Once(std::iter::once(Symbol { context, ident }))
        }
        Unresolved::Tuple(values) => Helper::Many(
            values
                .iter()
                .flat_map(|it| Box::new(collect_all_symbols(it))),
        ),
    }
}

fn resolve_values(
    values: Query<(NodeId<Value<Unresolved>>, &Value<Unresolved>)>,
) {
    let iter = values
        .iter()
        .flat_map(|(id, value)| collect_all_symbols(&value.inner).zip(repeat(id)))
        .map(|(a, b)| (b, a));

    for (id, symbol) in iter {}
}

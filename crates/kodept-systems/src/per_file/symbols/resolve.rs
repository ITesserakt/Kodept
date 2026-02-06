use crate::per_file::ast_normalization::InModule;
use crate::per_file::symbols::SymbolTable;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::name::Name;
use bevy_ecs::prelude::{Children, Entity};
use bevy_ecs::query::QueryData;
use bevy_ecs::system::Query;
use kodept_ast::prelude::{Erase, NodeId};
use kodept_ast_nodes::{Path, UnresolvedName, Value};
use kodept_core::try_port::Try;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::ControlFlow;

#[derive(Debug)]
enum Control<T> {
    Continue,
    Return(T),
    Break,
}

fn walk_up<'s, T: QueryData, R: Try>(
    mut ancestors: Query<'_, 's, (Option<&ChildOf>, T)>,
    start_id: impl Erase<Entity>,
    mut f: impl FnMut(T::Item<'_, 's>) -> R,
) -> Option<R::Output> {
    let Ok((mut maybe_parent, mut item)) = ancestors.get_mut(start_id.erase()) else {
        return None;
    };
    loop {
        match f(item).branch() {
            ControlFlow::Continue(r) => return Some(r),
            ControlFlow::Break(_) => {}
        }

        if let Some(ChildOf(parent)) = maybe_parent.cloned()
            && let Ok(next) = ancestors.get_mut(parent)
        {
            maybe_parent = next.0;
            item = next.1;
        } else {
            return None;
        }
    }
}

fn walk_down<'s, T: QueryData, R>(
    mut descendants: Query<'_, 's, (Option<&Children>, T)>,
    start_id: impl Erase<Entity>,
    mut f: impl FnMut(T::Item<'_, 's>) -> Control<R>,
) -> Option<R>
where
    R: Debug,
{
    let mut stack = VecDeque::from([start_id.erase()]);
    while let Some(next) = stack.pop_front() {
        if let Ok((children, item)) = descendants.get_mut(next) {
            match dbg!(f(item)) {
                Control::Continue => {}
                Control::Return(r) => return Some(r),
                Control::Break => continue,
            }

            stack.extend(children.iter().copied().flatten().copied());
        }
    }
    None
}

pub(super) fn resolve_values(
    values: Query<(
        NodeId<Value<UnresolvedName>>,
        &Value<UnresolvedName>,
        &InModule,
    )>,
    mut parent_symbol_tables: Query<(Option<&ChildOf>, Option<&SymbolTable>)>,
    mut children_symbol_tables: Query<(Option<&Children>, (&Name, &SymbolTable))>,
) {
    for (id, value, &InModule(module_id)) in values {
        let UnresolvedName { context, ident } = &value.inner;

        let result = match &context {
            Path {
                is_global: false,
                segments,
            } if segments.is_empty() => walk_up(parent_symbol_tables.reborrow(), id, |table| {
                let table = table?;
                let (decl_id, kind) = table.lookup(ident)?;
                Some((decl_id, kind.clone()))
            }),
            Path {
                is_global: true,
                segments,
            } if segments.is_empty() => {
                parent_symbol_tables
                    .get(module_id.entity())
                    .ok()
                    .and_then(|(_, table)| {
                        let table = table?;
                        let (decl_id, kind) = table.lookup(ident)?;
                        Some((decl_id, kind.clone()))
                    })
            }
            Path {
                is_global: true,
                segments,
            } => {
                let mut segments_iter = segments.iter();
                walk_down(
                    children_symbol_tables.reborrow(),
                    module_id,
                    |(name, table)| match segments_iter.next() {
                        None => {
                            let Some((decl_id, kind)) = table.lookup(ident) else {
                                return Control::Break;
                            };
                            Control::Return((decl_id, kind.clone()))
                        }
                        Some(expected_segment) if expected_segment.as_ref() == name.as_ref() => {
                            Control::Continue
                        }
                        Some(_) => Control::Break,
                    },
                )
            }
            Path {
                is_global: false, ..
            } => todo!(),
        };
    }
}

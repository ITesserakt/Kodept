use crate::per_file::ast_normalization::InModule;
use crate::per_file::symbols::{SymbolKind, SymbolTable};
use crate::source::collection::Reporter;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::name::Name;
use bevy_ecs::prelude::{Children, Entity};
use bevy_ecs::query::QueryData;
use bevy_ecs::system::{Commands, Query};
use kodept_ast::Str;
use kodept_ast::export::Component;
use kodept_ast::prelude::{Erase, NodeId};
use kodept_ast::properties::SourceSpan;
use kodept_ast_nodes::{Module, Path, UnresolvedName, Value};
use kodept_core::code_point::Span;
use kodept_core::try_port::Try;
use kodept_report_macros::Report;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::iter::Peekable;
use std::ops::ControlFlow;

#[derive(Debug, Component)]
#[component(immutable)]
pub(super) struct ResolvedTo(NodeId, SymbolKind);

#[derive(Debug, Report)]
#[severity("error")]
#[message("Cannot resolve reference `{}`", self.name)]
struct UnresolvedReference {
    name: Str,
    #[primary_label("not found in scope")]
    span: Span,
}

#[derive(Debug)]
enum Control<T> {
    Continue,
    Return(T),
    Break,
}

#[derive(Debug)]
enum Layered<T> {
    Layer,
    Value(T),
}

fn walk_up<'s, T: QueryData, R: Try>(
    mut ancestors: Query<'_, 's, (NodeId, Option<&ChildOf>, T)>,
    start_id: impl Erase<Entity>,
    mut f: impl FnMut(T::Item<'_, 's>) -> R,
) -> Option<(NodeId, R::Output)> {
    let Ok((mut id, mut maybe_parent, mut item)) = ancestors.get_mut(start_id.erase()) else {
        return None;
    };
    loop {
        match f(item).branch() {
            ControlFlow::Continue(r) => return Some((id, r)),
            ControlFlow::Break(_) => {}
        }

        if let Some(ChildOf(parent)) = maybe_parent.cloned()
            && let Ok(next) = ancestors.get_mut(parent)
        {
            id = next.0;
            maybe_parent = next.1;
            item = next.2;
        } else {
            return None;
        }
    }
}

fn walk_down<'s, T: QueryData, R>(
    mut descendants: Query<'_, 's, (NodeId, Option<&Children>, T)>,
    start_id: impl Erase<Entity>,
    mut f: impl FnMut(Layered<T::Item<'_, 's>>) -> Control<R>,
) -> Option<(NodeId, R)>
where
    R: Debug,
{
    let mut stack = VecDeque::from([Layered::Value(start_id.erase()), Layered::Layer]);
    loop {
        match stack.pop_front() {
            None => return None,
            Some(Layered::Layer) => {
                f(Layered::Layer);
                if !stack.is_empty() {
                    stack.push_back(Layered::Layer);
                }
            }
            Some(Layered::Value(next)) => match descendants.get_mut(next) {
                Ok((id, children, item)) => match f(Layered::Value(item)) {
                    Control::Continue => stack.extend(
                        children
                            .iter()
                            .flat_map(|it| it.iter())
                            .chain(Some(&id.entity()))
                            .map(|it| Layered::Value(*it)),
                    ),
                    Control::Return(r) => return Some((id, r)),
                    Control::Break => continue,
                },
                Err(_) => {}
            },
        }
    }
}

type AncestorsQuery<'w, 's> = Query<
    'w,
    's,
    (
        NodeId,
        Option<&'static ChildOf>,
        (Option<&'static Name>, Option<&'static SymbolTable>),
    ),
>;
type DescendantsQuery<'w, 's> = Query<
    'w,
    's,
    (
        NodeId,
        Option<&'static Children>,
        (&'static Name, &'static SymbolTable),
    ),
>;
type ResolutionResult = Option<(NodeId, NodeId, SymbolKind)>;

fn resolve_local_value(
    start_id: impl Erase,
    ident: &Str,
    ancestors: AncestorsQuery,
) -> ResolutionResult {
    let (table_node_id, (decl_id, kind)) = walk_up(ancestors, start_id, |(_, table)| {
        let table = table?;
        let (decl_id, kind) = table.lookup(ident)?;
        Some((decl_id, kind.clone()))
    })?;
    Some((table_node_id, decl_id, kind))
}

fn on_iteration<'a, I: Iterator<Item = &'a Str>>(
    mut segments_iter: Peekable<I>,
    ident: &Str,
    breaks_on: &mut usize,
) -> impl FnMut(Layered<(&Name, &SymbolTable)>) -> Control<(NodeId, SymbolKind)> {
    let mut index = 0;
    move |value| match value {
        Layered::Layer => {
            segments_iter.next();
            index += 1;
            Control::Continue
        }
        Layered::Value((name, table)) => match segments_iter.peek() {
            None => table
                .lookup(ident)
                .map(|it| Control::Return((it.0, it.1.clone())))
                .unwrap_or(Control::Break),
            Some(expected_segment) if expected_segment.as_ref() == name.as_ref() => {
                Control::Continue
            }
            Some(_) => {
                *breaks_on = index;
                Control::Break
            }
        },
    }
}

fn resolve_global_value_with_context(
    module_id: NodeId<Module>,
    context: &[Str],
    ident: &Str,
    descendants: DescendantsQuery,
) -> ResolutionResult {
    let segments_iter = context.iter().peekable();
    let mut breaks_on = 0;
    let result = walk_down(
        descendants,
        module_id,
        on_iteration(segments_iter, ident, &mut breaks_on),
    );
    result.map(|it| (it.0, it.1.0, it.1.1))
}

fn resolve_local_value_with_context(
    start_id: impl Erase,
    context: &[Str],
    ident: &Str,
    ancestors: AncestorsQuery,
    descendants: DescendantsQuery,
) -> ResolutionResult {
    let mut segments_iter = context.iter().peekable();
    let first = segments_iter.peek()?;
    let (start, _) = walk_up(ancestors, start_id, |(name, _)| {
        let name = name?;
        (name.as_ref() == first.as_ref()).then_some(())
    })?;
    let mut breaks_on = 0;
    let result = walk_down(
        descendants,
        start,
        on_iteration(segments_iter, ident, &mut breaks_on),
    );
    result.map(|it| (it.0, it.1.0, it.1.1))
}

pub(super) fn resolve_values(
    values: Query<(
        NodeId<Value<UnresolvedName>>,
        &Value<UnresolvedName>,
        &InModule,
        &SourceSpan,
    )>,
    parent_symbol_tables: AncestorsQuery,
    children_symbol_tables: DescendantsQuery,
    mut reporter: Reporter,
    mut commands: Commands,
) {
    for (id, value, &InModule(module_id), span) in values {
        let UnresolvedName { context, ident } = &value.inner;

        let result = match &context {
            Path {
                is_global: false,
                segments,
            } if segments.is_empty() => resolve_local_value(id, ident, parent_symbol_tables),
            Path {
                is_global: true,
                segments,
            } => resolve_global_value_with_context(
                module_id,
                segments,
                ident,
                children_symbol_tables,
            ),
            Path {
                is_global: false,
                segments,
            } => resolve_local_value_with_context(
                id,
                segments,
                ident,
                parent_symbol_tables,
                children_symbol_tables,
            ),
        };

        match result {
            None => {
                reporter.report(UnresolvedReference {
                    name: value.inner.ident.clone(),
                    span: span.0,
                });
            }
            Some((_, decl_id, kind)) => {
                commands
                    .entity(id.entity())
                    .insert(ResolvedTo(decl_id, kind));
            }
        };
    }
}

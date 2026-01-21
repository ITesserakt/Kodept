use super::*;
use bevy_ecs::prelude::*;
use kodept_ast::Str;
use kodept_ast::prelude::HierarchicalQuery;
use kodept_ast_nodes::v2::expression::BinExpr;
use kodept_ast_nodes::v2::properties::Rhs;
use kodept_ast_nodes::v2::term::Ref;
use kodept_ast_nodes::v2::term::ReferenceContext;

#[derive(Debug, Clone)]
struct RefView<'a> {
    context: &'a ReferenceContext,
    ident: &'a Str,
    expected_symbol_kinds: Cow<'static, [SymbolKind]>,
}

trait AsView {
    fn as_view(&self) -> RefView<'_>;
}

impl AsView for Ref {
    fn as_view(&self) -> RefView<'_> {
        RefView {
            context: &self.context,
            ident: &self.ident,
            expected_symbol_kinds: Cow::Borrowed(&[
                SymbolKind::Variable,
                SymbolKind::Parameter,
                SymbolKind::Function,
            ]),
        }
    }
}

impl AsView for Ty {
    fn as_view(&self) -> RefView<'_> {
        RefView {
            context: &self.context,
            ident: &self.ident,
            expected_symbol_kinds: Cow::Borrowed(&[SymbolKind::Type, SymbolKind::Const]),
        }
    }
}

pub(super) fn mark_refs_as_deferred(query: HierarchicalQuery<BinExpr, Ref, Rhs>) {
    for (_, _, parent, _) in query.iter() {
        if matches!(parent, BinExpr::Access) {
            todo!("Reference resolution in access expressions is not implemented yet");
        }
    }
}

#[allow(private_bounds)]
pub(super) fn start_resolution<T: ASTNode + AsView>(
    query: Populated<(Entity, &InScope, &T), (Without<DeferRefResolution>, Without<SymbolUsage>)>,
    mut writer: MessageWriter<ResolveRefAtMessage>,
) {
    let mut query_iter = query.into_iter();
    let into_message = move || {
        let (id, scope_id, node) = query_iter.next()?;
        let view = node.as_view();

        if !view.context.is_empty_local_context() {
            todo!("Contextual reference resolution is not implemented yet: {view:?}");
        }

        Some(ResolveRefAtMessage {
            ref_id: id,
            scope_id: scope_id.0,
        })
    };

    writer.write_batch(std::iter::from_fn(into_message));
}

#[allow(private_bounds)]
pub(super) fn process_resolve_messages(
    mut reader: MessageReader<ResolveRefAtMessage>,
    scopes: Query<(&SymbolTable, &Scoping, Option<&ChildOf>)>,
    refs: Query<(Entity, AnyOf<(&Ref, &Ty)>, &SourceSpan)>,
    mut commands: Commands,
) -> Result<(), Vec<UnresolvedReference>> {
    let mut errors = vec![];
    for msg in reader.read() {
        let Ok((ref_id, node, span)) = refs.get(msg.ref_id) else {
            continue;
        };
        let view = match node {
            (Some(x), None) => x.as_view(),
            (None, Some(x)) => x.as_view(),
            _ => unreachable!(),
        };

        let (table, _, scope_parent_id) = scopes.get(msg.scope_id).unwrap();
        if let Some(symbol) = view.find_symbol(table) {
            commands.entity(msg.ref_id).insert(SymbolUsage {
                kind: symbol.kind,
                symbol_scope_id: msg.scope_id,
            });
        } else if let Some(parent) = scope_parent_id {
            // bubble up
            commands.write_message(ResolveRefAtMessage {
                ref_id,
                scope_id: parent.0,
            });
            continue;
        } else {
            errors.push(UnresolvedReference {
                reference_name: Name::new(view.ident.clone()),
                reference_span: *span,
            });
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

impl RefView<'_> {
    fn find_symbol<'a>(&self, table: &'a SymbolTable) -> Option<&'a Symbol> {
        for kind in self.expected_symbol_kinds.iter() {
            let descriptor = SymbolDescriptorView {
                name: self.ident,
                kind,
            };
            if let Some(symbol) = table.symbols.get(&descriptor) {
                return Some(symbol);
            }
        }
        None
    }
}

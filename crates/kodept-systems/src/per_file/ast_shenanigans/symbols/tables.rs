use super::*;
use bevy_ecs::prelude::*;
use kodept_ast::prelude::ASTNode;
use kodept_ast::properties::{Node, RequireProperty, SourceSpan};

pub(super) fn spawn_symbol<T: ASTNode + RequireProperty<Name>>(
    kind: fn(&T) -> SymbolKind,
) -> impl FnMut(Populated<(Entity, &T), (With<Node>, Added<InScope>)>, MessageWriter<SpawnSymbolMessage>)
{
    move |query, mut writer| {
        writer.write_batch(query.into_iter().map(|it| SpawnSymbolMessage {
            entity: it.0,
            kind: kind(it.1),
        }));
    }
}

pub(super) fn populate_symbol_table(
    mut messages: MessageReader<SpawnSymbolMessage>,
    nodes: Query<(&Name, &InScope), With<Node>>,
    spans: Query<&SourceSpan, With<Node>>,
    mut tables: Query<(&mut SymbolTable, Option<&Name>, &Scope)>,
) -> Result<(), Vec<SymbolErrors>> {
    let mut errors = vec![];
    for spawned_symbol in messages.read() {
        let span = *spans.get(spawned_symbol.entity).unwrap();
        let Ok((name, scope_id)) = nodes.get(spawned_symbol.entity) else {
            errors.push(SymbolErrors::NameNotFound(span));
            continue;
        };
        let Ok((mut table, scope_name, scope)) = tables.get_mut(scope_id.0) else {
            errors.push(SymbolErrors::SymbolTableNotFound(scope_id.0));
            continue;
        };

        let descriptor = SymbolDescriptor {
            kind: spawned_symbol.kind,
            name: name.clone(),
        };
        match table.symbols.entry(descriptor) {
            Entry::Occupied(mut entry) if entry.get().bound_to == spawned_symbol.entity => {
                entry.get_mut().kind = spawned_symbol.kind;
            }
            Entry::Occupied(entry) => {
                errors.push(SymbolErrors::Duplicated {
                    scope_name: scope_name.cloned(),
                    symbol_name: name.clone(),
                    scope_span: *spans.get(scope.starts_from).unwrap(),
                    current_symbol_span: *spans.get(spawned_symbol.entity).unwrap(),
                    previous_symbol_span: *spans.get(entry.get().bound_to).unwrap(),
                });
                continue;
            }
            Entry::Vacant(entry) => {
                entry.insert(Symbol {
                    kind: spawned_symbol.kind,
                    bound_to: spawned_symbol.entity,
                });
            }
        };
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

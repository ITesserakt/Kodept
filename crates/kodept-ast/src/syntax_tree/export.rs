use crate::arity::{Optional, Plural, Singular};
use crate::prelude::AnyNodeRefItem;
use crate::properties::{Name, Node, Root};
use crate::relationship::{ArityValue, Contains, NodeRelationships};
use bevy_ecs::prelude::{
    Entity, EntityRef, InMut, IntoSystem, Query, ReadOnlySystem, Res, Single, SystemInput, With,
    World,
};
use std::io::Write;

impl super::storage::AST {
    pub fn export_dot_in<W: Write + 'static>(world: &World, writer: &mut W) -> std::io::Result<()> {
        run_readonly_system_with(world, extract_system, writer)
    }

    pub fn export_dot<W: Write + 'static>(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.interact().immediate_exclusive(|w| {
            Self::export_dot_in(w, writer)
        })
    }
}

fn label<W: Write>(buffer: &mut W, node: AnyNodeRefItem) -> std::io::Result<()> {
    writeln!(buffer, "object \"{}\" {{", node.id().to_bits())?;
    writeln!(buffer, "\tkind = {}", node.kind())?;
    writeln!(buffer, "\tspan = {}", node.span())?;
    writeln!(buffer, "\tindex = {}", node.id().index())?;
    writeln!(buffer, "\tgeneration = {}", node.id().generation())?;
    if let Some(name) = node.property::<Name>() {
        writeln!(buffer, "\tname = {}", name)?;
    }
    writeln!(buffer, "}}")?;

    Ok(())
}

fn run_readonly_system_with<M, In, Out>(
    world: &World,
    system: impl IntoSystem<In, Out, M, System: ReadOnlySystem<In = In, Out = Out>>,
    input: In::Inner<'_>,
) -> Out
where
    In: SystemInput,
{
    let mut system = IntoSystem::into_system(system);
    system.run_readonly(input, world).unwrap()
}

fn extract_system<W: Write>(
    InMut(buffer): InMut<W>,
    relationships: Res<NodeRelationships>,
    nodes: Query<EntityRef, With<Node>>,
    root: Single<Entity, With<Root>>,
) -> std::io::Result<()> {
    writeln!(buffer, "@startuml")?;

    let mut stack = vec![(None, *root)];

    while let Some((edge, current)) = stack.pop() {
        let Ok(this) = nodes.get(current) else {
            return Ok(());
        };

        for meta in relationships.into_iter() {
            let component_id = meta.forward_component_id();
            let Ok(ptr) = this.get_by_id(component_id) else {
                continue;
            };

            #[allow(unsafe_code)]
            match meta.arity() {
                ArityValue::Singular => {
                    for child in unsafe { ptr.deref::<Contains<(), Singular>>() } {
                        stack.push((Some((current, meta)), child))
                    }
                }
                ArityValue::Optional => {
                    for child in unsafe { ptr.deref::<Contains<(), Optional>>() } {
                        stack.push((Some((current, meta)), child))
                    }
                }
                ArityValue::Plural => {
                    for child in unsafe { ptr.deref::<Contains<(), Plural>>() } {
                        stack.push((Some((current, meta)), child))
                    }
                }
            }
        }

        label(buffer, AnyNodeRefItem::from_inner(this))?;
        let Some((parent_id, meta)) = edge else {
            continue;
        };

        if meta.is_empty_tag() {
            writeln!(
                buffer,
                "\t{} --> {}",
                parent_id.to_bits(),
                current.to_bits()
            )?;
        } else {
            writeln!(
                buffer,
                "\t{} --> {}: {}",
                parent_id.to_bits(),
                current.to_bits(),
                meta.tag_name()
            )?;
        }
    }

    writeln!(buffer, "@enduml")?;
    Ok(())
}

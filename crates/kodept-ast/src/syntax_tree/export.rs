use crate::arity::{Optional, Plural, Singular};
use crate::properties::{Name, Node, Root, SourceSpan};
use crate::relationship::{ArityValue, Contains, NodeRelationships};
use crate::resource::reflection::DebugRegistry;
use bevy_ecs::component::{ComponentId, ComponentInfo};
use bevy_ecs::prelude::{Entity, EntityRef, In, Query, With, World};
use std::convert::identity;
use std::io::Write;

impl super::storage::AST {
    pub fn export_dot_in<W: Write>(world: &World, writer: W) -> Option<std::io::Result<()>> {
        let relationships = world.resource();

        let mut nodes_query_state = world.try_query_filtered::<EntityRef, With<Node>>()?;
        let nodes_query = nodes_query_state.query(world);

        let mut root_query_state = world.try_query_filtered::<Entity, With<Root>>()?;
        let root_query = root_query_state.single(world).unwrap();

        Some(extract_system(
            In(writer),
            relationships,
            nodes_query,
            root_query,
            world,
        ))
    }
}

#[allow(unsafe_code)]
fn write_single_component(
    buffer: &mut impl Write,
    info: &ComponentInfo,
    node: &EntityRef,
    registry: Option<&DebugRegistry>,
) -> std::io::Result<()> {
    let ptr = node.get_by_id(info.id()).unwrap();
    let Some(type_id) = info.type_id() else {
        return write!(buffer, "\t\t\\t{} = <external component>", info.name());
    };

    let dyn_debug = match registry {
        Some(registry) => unsafe { registry.debug_dynamic(ptr, type_id) },
        None => unsafe { DebugRegistry::debug_dynamic_global(ptr, type_id) },
    };

    write!(buffer, "\t\t\\t{} = {:?}", info.name(), dyn_debug)
}

fn label<W: Write>(
    buffer: &mut W,
    node: EntityRef,
    world: &World,
    introspected_components: &[ComponentId],
) -> std::io::Result<()> {
    let (id, kind, span, name) = node.components::<(Entity, &Node, &SourceSpan, Option<&Name>)>();
    let all_components = world.inspect_entity(id).unwrap();
    let registry = world.get_resource::<DebugRegistry>();

    writeln!(buffer, "object \"{}\" {{", id.to_bits())?;
    writeln!(buffer, "\tkind = {}", kind)?;
    writeln!(buffer, "\tspan = {}", span)?;
    writeln!(buffer, "\tindex = {}", node.id().index())?;
    writeln!(buffer, "\tgeneration = {}", node.id().generation())?;
    if let Some(name) = name {
        writeln!(buffer, "\tname = {}", name)?;
    }
    write!(buffer, "\tother components = [\n")?;
    let mut first = true;
    for info in all_components {
        if introspected_components.contains(&info.id()) || &*info.name() == kind.kind {
            continue;
        }
        if first {
           first = false;
        } else {
            write!(buffer, ",\n")?;
        }
        write_single_component(buffer, info, &node, registry)?
    }
    writeln!(buffer, "\n\t]")?;
    writeln!(buffer, "}}")?;

    Ok(())
}

fn extract_system<W: Write>(
    In(mut buffer): In<W>,
    relationships: &NodeRelationships,
    nodes: Query<EntityRef, With<Node>>,
    root: Entity,
    world: &World,
) -> std::io::Result<()> {
    writeln!(buffer, "@startuml")?;

    let mut stack = vec![(None, root)];

    let introspected_components = [
        world.component_id::<Node>(),
        world.component_id::<SourceSpan>(),
        world.component_id::<Name>(),
    ]
    .into_iter()
    .filter_map(identity)
    .collect::<Vec<_>>();

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

        label(&mut buffer, this, world, &introspected_components)?;
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

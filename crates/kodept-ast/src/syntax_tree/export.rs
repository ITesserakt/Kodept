use crate::properties::{Node, Root};
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use bevy_hierarchy::{Children, HierarchyQueryExt};
use std::io::Write;

impl super::storage::AST {
    pub fn export_dot<W: Write + 'static>(&mut self, writer: W) -> std::io::Result<W> {
        self.world
            .run_system_once_with(writer, build_dot_system)
            .expect("Cannot run system")
    }
}

fn build_dot_system<W: Write>(
    In(mut buffer): In<W>,
    nodes: Populated<&Children>,
    nodes_with_children: Populated<(Entity, &Children), With<Node>>,
    kinds: Populated<&Node>,
    root: Single<Entity, With<Root>>,
) -> std::io::Result<W> {
    writeln!(buffer, "digraph {{")?;
    writeln!(
        buffer,
        "\t{} [ label = \"{} [{}v{}]\" ]",
        root.to_bits(),
        kinds.get(*root).unwrap().kind,
        root.index(),
        root.generation()
    )?;
    for node in nodes.iter_descendants(*root) {
        writeln!(
            buffer,
            "\t{} [ label = \"{} [{}v{}]\" ]",
            node.to_bits(),
            kinds.get(node).unwrap().kind,
            node.index(),
            node.generation()
        )?;
    }
    for (this, children) in nodes_with_children.iter() {
        for child in children {
            writeln!(buffer, "\t{} -> {}", this.to_bits(), child.to_bits())?;
        }
    }
    writeln!(buffer, "}}")?;
    Ok(buffer)
}

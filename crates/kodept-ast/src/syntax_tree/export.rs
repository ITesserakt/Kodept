use crate::properties::{Name, Node, Root};
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use std::io::Write;

impl super::storage::AST {
    pub fn export_dot<W: Write + 'static>(&mut self, writer: W) -> std::io::Result<W> {
        self.world
            .run_system_once_with(build_dot_system, writer)
            .expect("Cannot run system")
    }
}

fn build_dot_system<W: Write>(
    In(mut buffer): In<W>,
    nodes: Populated<&Children>,
    nodes_with_children: Populated<(Entity, &Children), With<Node>>,
    kinds: Populated<&Node>,
    names: Query<&Name>,
    root: Single<Entity, With<Root>>,
) -> std::io::Result<W> {
    writeln!(buffer, "digraph {{")?;
    writeln!(buffer, "\trankdir=\"LR\"")?;

    let root_name = names.get(*root).map_or("", |it| &*it);
    writeln!(
        buffer,
        "\t{} [ label = \"{} [{}v{}]|name: `{}`\", shape = \"record\" ]",
        root.to_bits(),
        kinds.get(*root).unwrap().kind,
        root.index(),
        root.generation(),
        root_name
    )?;
    for node in nodes.iter_descendants(*root) {
        let name = names.get(node).map_or("", |it| &*it);
        writeln!(
            buffer,
            "\t{} [ label = \"{} [{}v{}]|name: `{}`\", shape = \"record\" ]",
            node.to_bits(),
            kinds.get(node).unwrap().kind,
            node.index(),
            node.generation(),
            name
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

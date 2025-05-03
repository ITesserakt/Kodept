use crate::prelude::AnyNodeRefItem;
use crate::properties::Name;
use crate::query::AnyNodeQuery;
use std::io::Write;
use bevy_ecs::system::InMut;

impl super::storage::AST {
    pub fn export_dot<W: Write + 'static>(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.interact().immediate_with(writer, extract_system)
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

fn extract_system<W: Write>(
    InMut(buffer): InMut<W>,
    query: AnyNodeQuery,
) -> std::io::Result<()> {
    writeln!(buffer, "@startuml")?;

    for (edge, id) in query.iter_top_down_with_metadata() {
        let node = query.get(id).unwrap();
        label(buffer, node)?;
        if let Some((parent_id, meta)) = edge {
            if meta.is_empty_tag() {
                writeln!(buffer, "\t{} --> {}", parent_id.to_bits(), id.to_bits())?;
            } else {
                writeln!(
                    buffer,
                    "\t{} --> {}: {}",
                    parent_id.to_bits(),
                    id.to_bits(),
                    meta.tag_name()
                )?;
            }
        }
    }
    writeln!(buffer, "@enduml")?;
    Ok(())
}

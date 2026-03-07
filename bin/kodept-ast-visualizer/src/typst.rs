use crate::ExportControlEvent;
use crate::common::{State, StateOps};
use crate::utils::DebugAsDisplay;
use kodept_ast::relationship::RelationshipMetadata;
use kodept_ast::resource::reflection::DynDebug;
use kodept_ecs::archetype::Archetype;
use kodept_ecs::component::{ComponentId, ComponentInfo};
use kodept_ecs::entity::Entity;
use kodept_ecs::system::{Local, On};
use kodept_ecs::utils::DebugName;
use kodept_frontend::engine::{Engine, Plugin};
use kodept_systems::utils::ReportSystemEx;
use std::any::TypeId;
use std::borrow::Cow;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufWriter, Write};

pub(crate) struct TypstPlugin;

impl Plugin for TypstPlugin {
    fn build(self, engine: &mut Engine) {
        engine.add_observer(on_control_event.extract_reports());
    }
}

fn write_preamble(writer: &mut impl Write) -> std::io::Result<()> {
    write!(
        writer,
        r##"#import "@preview/cetz:0.4.2"

#set page(margin: 0.5em, width: auto, height: auto)

#let node(id, properties) = {{
  table(
    columns: 2,
    stroke: 0.5pt + white.darken(50%),
    align: center + horizon,
    table.cell(colspan: 2, fill: white.darken(30%))[*#id*],
    ..properties.map(it => {{
      (
        table.cell[#it.name],
        table.cell[#it.value],
      )
    }}).flatten()
  )
}}

#let nodes = (:)
#let edges = (:)
"##
    )?;
    Ok(())
}

fn sanitize(value: impl Display) -> String {
    let mut result = format!("{value}");
    let mut tail = result.as_str();
    let mut offset = 0;

    while let Some(brace_pos) = tail.find('"') {
        let replacement = match &tail[brace_pos..=brace_pos] {
            "\"" => "\\\"",
            _ => unreachable!(),
        };
        result.replace_range(offset + brace_pos..=offset + brace_pos, replacement);
        offset += brace_pos + replacement.len();
        tail = &result[offset..];
    }
    result
}

impl State<'_, '_, TypstState> {
    fn draw_edge(
        &self,
        from: Entity,
        to: Entity,
        meta: &RelationshipMetadata,
        writer: &mut impl Write,
    ) -> std::io::Result<()> {
        let tag = if meta.is_empty_tag() {
            Cow::Borrowed("none")
        } else {
            Cow::Owned(format!("\"{}\"", meta.tag_name()))
        };

        write!(
            writer,
            r##"#{{
    let children = edges.at("{}", default: ())
    children.push((tag: {tag}, child: "{}"))
    edges.insert("{}", children)
}}
"##,
            from.to_bits(),
            to.to_bits(),
            from.to_bits()
        )?;

        Ok(())
    }
}

type TypstState = ();
impl StateOps for State<'_, '_, TypstState> {
    fn get_entity_components(&self, archetype: &Archetype) -> impl Iterator<Item = ComponentId> {
        archetype.iter_components()
    }

    fn draw_node<'a>(
        &self,
        id: Entity,
        properties: impl Iterator<Item = (DynDebug<'a>, DebugName, bool, &'a ComponentInfo)>,
        buffer: &mut impl Write,
    ) -> std::io::Result<()> {
        write!(
            buffer,
            "#{{nodes.insert(\"{}\", node(\"{}\", (\n",
            id.to_bits(),
            id
        )?;
        for (debug_repr, name, mutable, type_id) in properties {
            let name = sanitize(name);
            let value = sanitize(DebugAsDisplay(debug_repr));
            write!(buffer, "\t(name: \"{name}\", value: \"{value}\"), \n")?;
        }
        write!(buffer, ")))}}\n")?;
        Ok(())
    }
}

fn on_control_event(
    control: On<ExportControlEvent>,
    mut state: State<TypstState>,
    mut output_buffer: Local<Option<BufWriter<File>>>,
) -> std::io::Result<()> {
    match (control.event(), &mut *output_buffer) {
        (ExportControlEvent::Start, buffer) => {
            let mut file = state.provide_output_file("typ")?;

            write_preamble(&mut file)?;
            *buffer = Some(BufWriter::new(file));
        }
        (ExportControlEvent::Finish, Some(writer)) => {
            writer.flush()?;
        }
        (ExportControlEvent::Root(id), Some(writer)) => {
            let entity = state.nodes.get(*id).expect("Cannot get node");
            write!(writer, "#let root_node_id = \"{}\"\n", id.to_bits())?;
            state.draw_node(entity, writer)?;
        }
        (
            ExportControlEvent::Inner {
                parent_id,
                metadata,
                this_id,
            },
            Some(writer),
        ) => {
            let entity = state.nodes.get(*this_id).expect("Cannot get node");
            state.draw_node(entity, writer)?;
            state.draw_edge(*parent_id, *this_id, metadata, writer)?;
        }
        _ => {}
    }
    Ok(())
}

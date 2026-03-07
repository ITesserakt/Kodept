use crate::ExportControlEvent;
use crate::common::{State, StateOps};
use crate::utils::DebugAsDisplay;
use kodept_ast::resource::reflection::DynDebug;
use kodept_ecs::archetype::Archetype;
use kodept_ecs::component::{ComponentId, ComponentInfo};
use kodept_ecs::entity::Entity;
use kodept_ecs::exported::bevy_ecs::entity::EntityHashMap;
use kodept_ecs::system::{Local, On};
use kodept_ecs::utils::DebugName;
use kodept_frontend::engine::{Engine, Plugin};
use kodept_systems::utils::ReportSystemEx;
use std::fmt::{Display, Formatter};
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
        r##"#import "@preview/tdtr:0.5.0"

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
        table.cell(align: left + horizon)[#it.value],
      )
    }}).flatten()
  )
}}

#tdtr.tidy-tree-graph(
    node-stroke: 0pt,
)[
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
        write!(buffer, "#{{node(\"{}\", (\n", id)?;
        for (debug_repr, name, mutable, type_id) in properties {
            let name = sanitize(name);
            let value = sanitize(DebugAsDisplay(debug_repr));
            write!(buffer, "\t(name: \"{name}\", value: \"{value}\"), \n")?;
        }
        write!(buffer, "))}}\n")?;
        Ok(())
    }
}

struct RootId(Entity);

impl Default for RootId {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

struct RepeatedDisplay<'a, T: ?Sized>(&'a T, usize);

impl<T: Display + ?Sized> Display for RepeatedDisplay<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.1 {
            T::fmt(self.0, f)?;
        }
        Ok(())
    }
}

fn on_control_event(
    control: On<ExportControlEvent>,
    mut state: State<TypstState>,
    mut output_buffer: Local<Option<BufWriter<File>>>,
    mut root: Local<RootId>,
    mut nodes: Local<EntityHashMap<Vec<u8>>>,
    mut edges: Local<EntityHashMap<Vec<(Entity, Option<&'static str>)>>>,
) -> std::io::Result<()> {
    match (control.event(), &mut *output_buffer) {
        (ExportControlEvent::Start, buffer) => {
            let file = state.provide_output_file("typ")?;
            *buffer = Some(BufWriter::new(file));
        }
        (ExportControlEvent::Finish, Some(writer)) => {
            write_preamble(writer)?;

            let mut stack = Vec::from([(root.0, 0, None)]);
            while let Some((id, depth, tag)) = stack.pop() {
                let this = nodes.get(&id).unwrap();
                let indent = RepeatedDisplay(" ", depth);
                match tag {
                    Some(tag) => {
                        writeln!(writer, "{indent}+ #[{tag}]")?;
                        write!(writer, "{indent}- ")?;
                    }
                    _ => write!(writer, "{indent}- ")?,
                };
                writer.write_all(this)?;
                writeln!(writer)?;

                for (child_id, tag) in edges.get(&id).into_iter().flatten() {
                    stack.push((*child_id, depth + 1, *tag));
                }
            }

            writeln!(writer, "]")?;
            writer.flush()?;
        }
        (ExportControlEvent::Root(id), _) => {
            root.0 = *id;
            let entity = state.nodes.get(*id).expect("Cannot get node");
            let mut node_string = Vec::new();
            state.draw_node(entity, &mut node_string)?;
            nodes.insert(*id, node_string);
        }
        (
            ExportControlEvent::Inner {
                parent_id,
                metadata,
                this_id,
            },
            _,
        ) => {
            let entity = state.nodes.get(*this_id).expect("Cannot get node");
            let mut node_string = Vec::new();
            state.draw_node(entity, &mut node_string)?;
            nodes.insert(*this_id, node_string);

            let tag_string = (!metadata.is_empty_tag()).then_some(metadata.tag_name());
            edges
                .entry(*parent_id)
                .or_default()
                .push((*this_id, tag_string));
        }
        _ => {}
    }
    Ok(())
}

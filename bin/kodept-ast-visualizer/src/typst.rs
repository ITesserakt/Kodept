use crate::ExportControlEvent;
use crate::common::{Config, State, StateOps};
use crate::utils::{DebugAsDisplay, NonVerboseComponents};
use kodept_ast::properties::Node;
use kodept_ast::resource::reflection::DynDebug;
use kodept_ecs::archetype::Archetype;
use kodept_ecs::component::{ComponentId, ComponentInfo};
use kodept_ecs::entity::{Entity, EntityHashMap};
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::system::{Local, On, Query, Res, SystemParam};
use kodept_ecs::utils::{DebugName, ShortName};
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

#let node(id, properties, size: none) = {{
  table(
    columns: if size != none {{ 3 }} else {{ 2 }},
    stroke: 0.5pt + white.darken(50%),
    align: center + horizon,
    table.cell(colspan: 2, fill: color.rgb(0, 0, 0, 33%))[*#id*],
    ..(if size != none {{ (table.cell[*#size*], ) }} else {{ () }}),
    ..properties.map(it => {{
      let name = if it.is_mutable {{ [_#{{it.name}}_] }} else {{ [#it.name] }}
      let fill = if it.is_unknown {{ color.rgb(0, 0, 0, 10%) }} else {{ none }}
      (
        table.cell(fill: fill, name),
        table.cell(fill: fill, align: left + horizon)[#it.value],
        ..(if it.at("size", default: none) != none {{ (table.cell(fill: fill, $#it.size$), ) }} else {{ () }})
      )
    }}).flatten()
  )
}}

#tdtr.tidy-tree-graph(
    node-stroke: 0pt,
    node-inset: 0pt,
)[
"##
    )?;
    Ok(())
}

fn sanitize(value: impl Display, max_len: usize) -> String {
    let mut result = format!("{value}");
    let mut tail = result.as_str();
    let mut offset = 0;

    while let Some(quote_pos) = tail.find('"') {
        let replacement = "\\\"";
        result.replace_range(offset + quote_pos..=offset + quote_pos, replacement);
        offset += quote_pos + replacement.len();
        tail = &result[offset..];
    }

    let mut lines = String::new();
    let mut first = true;
    for line in result.lines() {
        if first {
            first = false;
        } else {
            lines.push_str("\\n");
        }
        if line.len() >= max_len {
            lines.push_str(&line[..max_len]);
            lines.push_str("...");
        } else {
            lines.push_str(&line);
        }
    }
    lines
}

#[derive(SystemParam)]
struct TypstState<'w, 's> {
    config: Res<'w, Config>,
    non_verbose_components: NonVerboseComponents<'w, 's>,
    archetypes: Query<'w, 's, &'static Archetype>,
}

impl StateOps for State<'_, '_, TypstState<'_, '_>> {
    fn get_entity_components(&self, archetype: &Archetype) -> impl Iterator<Item = ComponentId> {
        let non_verbose_components = self.non_verbose_components.get();
        archetype
            .iter_components()
            .filter(move |it| self.config.verbose || non_verbose_components.contains(it))
    }

    fn draw_node<'a>(
        &self,
        id: Entity,
        properties: impl Iterator<Item = (DynDebug<'a>, DebugName, &'a ComponentInfo)>,
        buffer: &mut impl Write,
    ) -> std::io::Result<()> {
        let total_components_size = self
            .archetypes
            .get(id)
            .into_iter()
            .flat_map(|it| it.iter_components())
            .filter_map(|it| self.components.get_info(it))
            .map(|it| it.layout().size())
            .sum::<usize>();

        write!(buffer, "#{{node(\"{}\", (\n", id)?;
        for (debug_repr, name, info) in properties {
            if !debug_repr.is_known() && !self.config.show_unknown_components {
                continue;
            }
            if info.layout().size() == 0 && !self.config.show_zst_components {
                continue;
            }

            let name = match self.config.long_type_paths {
                false => sanitize(name.shortname(), usize::MAX),
                true => sanitize(name, usize::MAX),
            };
            let value = match info.id() == self.non_verbose_components.node.get() {
                false if self.config.multiline => {
                    let render = sanitize(DebugAsDisplay::new(debug_repr), self.config.max_length);
                    (render.len() < self.config.max_length)
                        .then_some(render)
                        .unwrap_or_else(|| sanitize(DebugAsDisplay::fancy(debug_repr), usize::MAX))
                }
                false => sanitize(DebugAsDisplay::new(debug_repr), self.config.max_length),
                true if self.config.long_type_paths => {
                    sanitize(DebugAsDisplay::new(debug_repr), usize::MAX)
                }
                true => {
                    #[allow(unsafe_code)]
                    let value = unsafe { debug_repr.into_inner().deref::<Node>() };
                    sanitize(ShortName::from(value.name), usize::MAX)
                }
            };

            write!(
                buffer,
                "\t(name: \"{name}\", value: \"{value}\", is_mutable: {}, is_unknown: {}",
                info.mutable(),
                !debug_repr.is_known(),
            )?;
            if self.config.show_components_size {
                write!(buffer, "size: {}", info.layout().size())?;
            }
            writeln!(buffer, "), ")?;
        }
        if !self.config.show_components_size {
            writeln!(buffer, "))}}")?;
        } else {
            writeln!(buffer, "), size: {})}}", total_components_size)?;
        }
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
                match (tag, state.config.long_type_paths) {
                    (Some(tag), true) => {
                        writeln!(writer, "{indent}+ #[{tag}]")?;
                        write!(writer, "{indent}- ")?;
                    }
                    (Some(tag), false) => {
                        writeln!(writer, "{indent}+ #[{}]", ShortName::from(tag))?;
                        write!(writer, "{indent}- ")?;
                    }
                    (None, _) => write!(writer, "{indent}- ")?,
                };
                writer.write_all(this)?;
                writeln!(writer)?;

                stack.extend(
                    edges
                        .get(&id)
                        .into_iter()
                        .flatten()
                        .map(|it| (it.0, depth + 1, it.1))
                        .rev(),
                );
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

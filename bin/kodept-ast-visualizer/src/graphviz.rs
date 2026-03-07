use crate::ExportControlEvent;
use crate::common::{State, StateOps};
use crate::graphviz::helpers::{DebugAsDisplay, row, sanitize, table};
use crate::utils::NonVerboseComponents;
use clap::Args;
use kodept_ast::properties::Node;
use kodept_ast::relationship::RelationshipMetadata;
use kodept_ast::resource::reflection::DynDebug;
use kodept_ecs::archetype::Archetype;
use kodept_ecs::component::{ComponentId, ComponentInfo};
use kodept_ecs::entity::Entity;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::resource::Resource;
use kodept_ecs::system::{Local, On, Query, Res, SystemParam};
use kodept_ecs::utils::{DebugName, ShortName};
use kodept_ecs::world::EntityRef;
use kodept_frontend::engine::{Engine, Plugin};
use kodept_systems::utils::ReportSystemEx;
use std::any::TypeId;
use std::fs::File;
use std::io::{BufWriter, Write};

pub(crate) struct GraphvizPlugin;

#[derive(Debug, Resource, Args, Clone)]
pub(crate) struct Config {
    /// Specifies amount of information to show for each AST node
    #[arg(short, long, action, default_value_t = false)]
    verbose: bool,
    /// Specifies whether components with no debug representation should appear in tables
    #[arg(short = 'u', long = "unknown", action, default_value_t = false)]
    show_unknown_components: bool,
    /// Specifies whether components with zero size (ZST) should appear in tables
    #[arg(short = 'z', long = "zst", action, default_value_t = false)]
    show_zst_components: bool,
    /// Adds a table column with components' size in bytes
    #[arg(long = "size", action, default_value_t = false)]
    show_components_size: bool,
    /// Do not trim type path at component names
    #[arg(short = 'l', long, action, default_value_t = false)]
    long_type_paths: bool,
    /// Specifies maximum length of a component value
    #[arg(short, long, default_value_t = 50)]
    max_length: usize,
    /// Print component values with line breaks
    #[arg(long, default_value_t = false)]
    multiline: bool,
}

impl Plugin for GraphvizPlugin {
    fn build(self, engine: &mut Engine) {
        engine.add_observer(on_control_event.extract_reports());
    }
}

mod helpers {
    use std::fmt::{Debug, Display, Formatter};
    use std::io::Write;

    pub(super) struct DebugAsDisplay<T> {
        value: T,
        fancy: bool,
    }

    impl<T: Debug> Display for DebugAsDisplay<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(self, f)
        }
    }

    impl<T: Debug> Debug for DebugAsDisplay<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if self.fancy {
                write!(f, "{:#?}", self.value)
            } else {
                write!(f, "{:?}", self.value)
            }
        }
    }

    impl<T> DebugAsDisplay<T> {
        pub(super) fn new(value: T) -> Self {
            Self {
                fancy: false,
                value,
            }
        }

        pub(super) fn fancy(value: T) -> Self {
            Self { value, fancy: true }
        }
    }

    pub(super) fn table<W, T, U>(
        writer: &mut W,
        properties: impl IntoIterator<Item = (T, U)>,
        f: impl FnOnce(&mut W) -> std::io::Result<()>,
    ) -> std::io::Result<()>
    where
        W: Write,
        T: Display,
        U: Display,
    {
        write!(writer, "<table ")?;
        for (name, value) in properties {
            property(writer, name, value)?
        }
        write!(writer, ">")?;
        f(writer)?;
        write!(writer, "</table>")
    }

    pub(super) fn property(
        writer: &mut impl Write,
        name: impl Display,
        value: impl Display,
    ) -> std::io::Result<()> {
        write!(writer, "{name}=\"{value}\" ")
    }

    pub(super) fn row<W: Write>(
        writer: &mut W,
        f: impl FnOnce(&mut W) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        write!(writer, "<tr>")?;
        f(writer)?;
        write!(writer, "</tr>")
    }

    pub(super) fn sanitize(value: impl Display, max_len: usize) -> String {
        let mut result = format!("{value}");
        let mut tail = result.as_str();
        let mut offset = 0;

        while let Some(brace_pos) = tail.find(['<', '>']) {
            let replacement = match &tail[brace_pos..=brace_pos] {
                "<" => "&lt;",
                ">" => "&gt;",
                _ => unreachable!(),
            };
            result.replace_range(offset + brace_pos..=offset + brace_pos, replacement);
            offset += brace_pos + replacement.len();
            tail = &result[offset..];
        }

        let mut lines = String::new();
        let mut first = true;
        for line in result.lines() {
            if first {
                first = false;
            } else {
                lines.push_str("<BR/>");
            }
            let line = line.replace("    ", "&nbsp;");
            if line.len() >= max_len {
                lines.push_str(&line[..max_len]);
                lines.push_str("...");
            } else {
                lines.push_str(&line);
            }
        }
        lines
    }
}

fn draw_edge(
    entity: EntityRef,
    parent: EntityRef,
    meta: &RelationshipMetadata,
    config: &Config,
    buffer: &mut impl Write,
) -> std::io::Result<()> {
    match (config.long_type_paths, meta.is_empty_tag()) {
        (_, true) => writeln!(
            buffer,
            "\t\"{}\" -> \"{}\"",
            parent.id().to_bits(),
            entity.id().to_bits()
        ),
        (true, _) => writeln!(
            buffer,
            "\t\"{}\" -> \"{}\" [ label = \"{}\" ]",
            parent.id().to_bits(),
            entity.id().to_bits(),
            meta.tag_name()
        ),
        (false, _) => writeln!(
            buffer,
            "\t\"{}\" -> \"{}\" [ label = \"{}\" ]",
            parent.id().to_bits(),
            entity.id().to_bits(),
            ShortName::from(meta.tag_name())
        ),
    }
}

#[derive(SystemParam)]
struct GraphvizState<'w, 's> {
    non_verbose_components: NonVerboseComponents<'w, 's>,
    config: Res<'w, Config>,
    archetypes: Query<'w, 's, &'static Archetype>,
}

impl StateOps for State<'_, '_, GraphvizState<'_, '_>> {
    fn get_entity_components(&self, archetype: &Archetype) -> impl Iterator<Item = ComponentId> {
        let non_verbose_components = self.non_verbose_components.get();

        archetype
            .components()
            .into_iter()
            .copied()
            .filter(move |it| self.config.verbose || non_verbose_components.contains(it))
    }

    fn draw_node<'a>(
        &self,
        id: Entity,
        properties: impl Iterator<Item = (DynDebug<'a>, DebugName, bool, &'a ComponentInfo)>,
        buffer: &mut impl Write,
    ) -> std::io::Result<()> {
        let total_node_size = self
            .archetypes
            .get(id)
            .map(|it| it.iter_components())
            .into_iter()
            .flatten()
            .filter_map(|it| self.components.get_info(it))
            .map(|it| it.layout().size())
            .sum::<usize>();

        write!(buffer, "\t\"{}\" [ label=<", id.to_bits())?;
        table(
            buffer,
            [("border", 0), ("cellborder", 1), ("cellspacing", 0)],
            move |buffer| {
                row(buffer, |buffer| {
                    write!(
                        buffer,
                        "<td bgcolor=\"#00000033\" colspan=\"2\"><b>{}</b></td>",
                        id
                    )?;

                    if self.config.show_components_size {
                        write!(buffer, "<td>{}</td>", total_node_size)?;
                    }
                    Ok(())
                })?;

                for (repr, name, is_mutable, info) in properties {
                    if !repr.is_known() && !self.config.show_unknown_components {
                        continue;
                    }
                    let name = match self.config.long_type_paths {
                        true => sanitize(name, self.config.max_length),
                        false => sanitize(name.shortname(), self.config.max_length),
                    };
                    let is_known = repr.is_known();
                    let is_node = info.type_id().is_some_and(|it| it == TypeId::of::<Node>());
                    let repr = match (self.config.long_type_paths, is_node) {
                        (false, true) => {
                            #[allow(unsafe_code)]
                            let node = unsafe { repr.into_inner().deref::<Node>() };
                            let path = ShortName::from(node.name);
                            sanitize(path, self.config.max_length)
                        }
                        _ => {
                            let render =
                                sanitize(DebugAsDisplay::new(repr), self.config.max_length);
                            if render.len() >= self.config.max_length && self.config.multiline {
                                sanitize(DebugAsDisplay::fancy(repr), usize::MAX)
                            } else {
                                render
                            }
                        }
                    };

                    row(buffer, |buffer| {
                        if is_known {
                            write!(buffer, "<td>")?;
                        } else {
                            write!(buffer, "<td bgcolor=\"#0000000a\">")?;
                        }
                        if is_mutable {
                            write!(buffer, "<i>{name}</i></td>")?;
                        } else {
                            write!(buffer, "{name}</td>")?;
                        }
                        write!(buffer, "<td>{repr}</td>")?;
                        if self.config.show_components_size {
                            write!(buffer, "<td>{}</td>", info.layout().size())?;
                        }
                        Ok(())
                    })?;
                }

                Ok(())
            },
        )?;
        writeln!(buffer, ">, shape=plain ]")
    }
}

fn on_control_event(
    control: On<ExportControlEvent>,
    mut state: State<GraphvizState>,
    mut output_buffer: Local<Option<BufWriter<File>>>,
) -> std::io::Result<()> {
    match (control.event(), &mut *output_buffer) {
        (ExportControlEvent::Start, buffer) => {
            let mut file = state.provide_output_file("dot")?;

            writeln!(file, "digraph g {{")?;
            writeln!(file, "\tgraph [ rankdir = \"TD\", pad = 0.1 ]")?;
            writeln!(
                file,
                "\tnode [ style=filled, shape=rect, pencolor=\"#00000044\", color=\"#00000000\",  fontname=\"Helvetica,Arial,sans-serif\", shape=plaintext ]"
            )?;

            *buffer = Some(BufWriter::new(file));
        }
        (ExportControlEvent::Finish, buffer) => {
            if let Some(mut writer) = buffer.take() {
                writeln!(writer, "}}")?;

                writer.flush()?;
            }
        }
        (&ExportControlEvent::Root(id), Some(buffer)) => {
            let entity = state.nodes.get(id).expect("Cannot draw node");
            state.draw_node(entity, buffer)?;
        }
        (
            ExportControlEvent::Inner {
                parent_id,
                metadata,
                this_id,
            },
            Some(buffer),
        ) => {
            let [parent, entity] = state
                .nodes
                .get_many([*parent_id, *this_id])
                .expect("Cannot get node");
            state.draw_node(entity, buffer)?;
            draw_edge(entity, parent, metadata, &*state.config, buffer)?;
        }
        _ => {}
    }

    Ok(())
}

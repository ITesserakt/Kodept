use crate::ExportControlEvent;
use crate::graphviz::helpers::{DebugAsDisplay, cell, row, sanitize, table};
use bevy_ecs::component::Components;
use bevy_ecs::prelude::{EntityRef, Local, Name, On, Query, Res, Resource, With};
use bevy_utils::prelude::ShortName;
use clap::Args;
use kodept_ast::properties::{Node, SourceSpan};
use kodept_ast::relationship::RelationshipMetadata;
use kodept_ast::resource::reflection::DebugRegistry;
use kodept_frontend::Either;
use kodept_frontend::engine::{Engine, Plugin};
use kodept_systems::configs::OutputDirectory;
use kodept_systems::source::collection::SourceView;
use kodept_systems::utils::ReportSystemEx;
use std::any::TypeId;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::id;

pub(crate) struct GraphvizPlugin;

#[derive(Debug, Resource, Args, Clone)]
pub(crate) struct Config {
    /// Specifies amount of information to show for each AST node
    #[arg(short, long, action, default_value_t = false)]
    verbose: bool,
    /// Specifies whether components with no debug representation should appear in tables
    #[arg(short = 'u', long = "unknown", action, default_value_t = false)]
    show_unknown_components: bool,
    /// Do not trim type path at component names
    #[arg(short = 'l', long, action, default_value_t = false)]
    long_type_paths: bool,
}

impl Plugin for GraphvizPlugin {
    fn build(self, engine: &mut Engine) {
        engine.add_observer(on_control_event.extract_reports());
    }
}

mod helpers {
    use std::fmt::{Debug, Display, Formatter};
    use std::io::Write;

    #[repr(transparent)]
    pub(super) struct DebugAsDisplay<T>(pub T);

    impl<T: Debug> Display for DebugAsDisplay<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            T::fmt(&self.0, f)
        }
    }

    impl<T: Debug> Debug for DebugAsDisplay<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            T::fmt(&self.0, f)
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

    pub(super) fn cell<W: Write>(
        writer: &mut W,
        f: impl FnOnce(&mut W) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        write!(writer, "<td>")?;
        f(writer)?;
        write!(writer, "</td>")
    }

    pub(super) fn sanitize(value: impl Display) -> String {
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
        result
    }
}

fn draw_node(
    entity: EntityRef,
    buffer: &mut impl Write,
    debug_registry: Option<&DebugRegistry>,
    components: &Components,
    config: &Config,
) -> std::io::Result<()> {
    let non_verbose_components = [
        components.component_id::<Node>(),
        components.component_id::<SourceSpan>(),
        components.component_id::<Name>(),
    ];

    let all_components = entity.archetype().components();

    // SAFETY: `components_value` enumerates values in the same order as `all_components`.
    //         That means type id must equal to the original type.
    #[allow(unsafe_code)]
    let components_debug_repr = all_components
        .into_iter()
        .filter(|id| config.verbose || non_verbose_components.contains(&Some(**id)))
        .map(|id| (entity.get_by_id(*id), id))
        .filter_map(|(value, &id)| Some((value.ok()?, components.get_info(id)?)))
        .filter_map(|(value, info)| {
            let type_id = info.type_id()?;
            let debug_repr = match debug_registry {
                Some(registry) => unsafe { registry.debug_dynamic(value, type_id) },
                None => unsafe { DebugRegistry::debug_dynamic_global(value, type_id) },
            };
            Some((debug_repr, info.name(), info.mutable(), type_id))
        });

    write!(buffer, "\t\"{}\" [ label=<", entity.id().to_bits())?;
    table(
        buffer,
        [("border", 0), ("cellborder", 1), ("cellspacing", 0)],
        move |buffer| {
            row(buffer, |buffer| {
                write!(
                    buffer,
                    "<td bgcolor=\"#00000033\" colspan=\"2\"><b>{}</b></td>",
                    entity.id()
                )
            })?;

            for (repr, name, is_mutable, type_id) in components_debug_repr {
                if !repr.is_known() && !config.show_unknown_components {
                    continue;
                }
                let name = match config.long_type_paths {
                    true => sanitize(name),
                    false => sanitize(name.shortname()),
                };
                let is_known = repr.is_known();
                let repr = match (config.long_type_paths, type_id == TypeId::of::<Node>()) {
                    (false, true) => {
                        #[allow(unsafe_code)]
                        let node = unsafe { repr.into_inner().deref::<Node>() };
                        let path = node.kind.shortname();
                        Either::Right(path)
                    }
                    _ => Either::Left(DebugAsDisplay(repr)),
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
                    cell(buffer, |buffer| write!(buffer, "{repr}"))?;
                    Ok(())
                })?;
            }

            Ok(())
        },
    )?;
    writeln!(buffer, ">, shape=plain ]")
}

fn draw_edge(
    entity: EntityRef,
    parent: EntityRef,
    meta: &RelationshipMetadata,
    config: &Config,
    buffer: &mut impl Write,
) -> std::io::Result<()> {
    let edge_label = match (config.long_type_paths, meta.is_empty_tag()) {
        (_, true) => Either::Left(""),
        (true, _) => Either::Left(meta.tag_name()),
        (false, _) => Either::Right(ShortName::from(meta.tag_name())),
    };
    writeln!(
        buffer,
        "\t\"{}\" -> \"{}\" [ label = \"{}\" ]",
        parent.id().to_bits(),
        entity.id().to_bits(),
        edge_label
    )?;
    Ok(())
}

fn on_control_event(
    control: On<ExportControlEvent>,
    output: Res<OutputDirectory>,
    source: Res<SourceView>,
    nodes: Query<EntityRef, With<Node>>,
    components: &Components,
    debug_registry: Option<Res<DebugRegistry>>,
    config: Res<Config>,
    mut output_buffer: Local<Option<BufWriter<File>>>,
) -> std::io::Result<()> {
    match (control.event(), &mut *output_buffer) {
        (ExportControlEvent::Start, buffer) => {
            output.create_missing_folders()?;
            let descriptor = source.describe();
            let filename = descriptor.name();
            let filepath = output.get_path_for_source(filename, "dot")?;

            let mut file = File::create(filepath)?;
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
            let entity = nodes.get(id).expect("Cannot get node");
            draw_node(
                entity,
                buffer,
                debug_registry.as_deref(),
                components,
                &*config,
            )?;
        }
        (
            ExportControlEvent::Inner {
                parent_id,
                metadata,
                this_id,
            },
            Some(buffer),
        ) => {
            let [parent, entity] = nodes
                .get_many([*parent_id, *this_id])
                .expect("Cannot get node");
            draw_node(
                entity,
                buffer,
                debug_registry.as_deref(),
                components,
                &*config,
            )?;
            draw_edge(entity, parent, metadata, &*config, buffer)?;
        }
        _ => {}
    }

    Ok(())
}

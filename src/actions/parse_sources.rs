use crate::cli::configs::ParsingConfig;
use bevy_ecs::prelude::{Component, Entity, ParallelCommands, Populated, Res};
use bevy_ecs::query::Without;
use kodept::codespan_settings::ReportEmitted;
use kodept::source_files::SourceView;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_frontend::frontend::Frontend;
use kodept_frontend::plugin::Plugin;
use kodept_parse::error::{ParseError, ParseErrors};
use kodept_report::error::report::{Label, Severity};
use kodept_report::error::Diagnostic;
use std::borrow::Cow;
use std::fmt::Display;
use std::fmt::Write;

pub struct ParseSourcesPlugin;

impl Plugin for ParseSourcesPlugin {
    fn build(self, app: &mut Frontend) {
        app.add_systems(parse_sources_system);
    }
}

#[derive(Component)]
struct Parsed;

fn parse_sources_system(
    query: Populated<(Entity, &SourceView), Without<Parsed>>,
    commands: ParallelCommands,
    config: Res<ParsingConfig>,
) {
    query.par_iter().for_each(|(entity, it)| {
        let rlt = config.build_rlt(&*it);
        match rlt {
            Ok(rlt) => {
                commands.command_scope(|mut commands| {
                    commands
                        .entity(entity)
                        .insert((Parsed, SyntaxResolver::empty(rlt)));
                });
            }
            Err(error) => {
                let diagnostics = to_diagnostics(error);
                commands.command_scope(|mut commands| {
                    for diagnostic in diagnostics {
                        commands.send_event(ReportEmitted::new(*it.id, diagnostic));
                    }
                    commands.entity(entity).insert(Parsed);
                });
            }
        }
    });
}

fn to_diagnostic<A: Display>(error: ParseError<A>) -> Diagnostic {
    let (expected, actual, location, hints) = match error {
        ParseError::ExpectedInstead {
            expected,
            actual,
            location,
            hints,
        } => (expected, Some(actual), location, hints),
        ParseError::ExpectedNotEOF {
            expected,
            location,
            hints,
        } => (expected, None, location, hints),
    };

    let diagnostic = if expected.is_empty() {
        let actual = actual
            .map(|it| Cow::Owned(it.to_string()))
            .unwrap_or(Cow::Borrowed("EOF"));

        Diagnostic::new(Severity::Error)
            .with_message(format!("Unexpected {actual}"))
            .with_label(Label::primary("here", location.in_code))
    } else if let Some(actual) = actual {
        let exp_msg = expected_to_string(expected);

        Diagnostic::new(Severity::Error)
            .with_message(format!("Expected {exp_msg}, got {actual}"))
            .with_label(Label::primary("here", location.in_code))
    } else {
        let exp_msg = expected_to_string(expected);

        Diagnostic::new(Severity::Error)
            .with_message(format!("Expected {exp_msg} after, got EOF"))
            .with_label(Label::primary("here", location.in_code))
    };

    hints
        .into_iter()
        .fold(diagnostic, |acc, next| acc.with_note(next))
}

fn expected_to_string(mut expected: Vec<Cow<'static, str>>) -> Cow<'static, str> {
    let Some(last_expected) = expected.pop() else {
        return Cow::Borrowed("");
    };

    if let Some(last) = expected.pop() {
        let mut result = String::new();
        for item in expected {
            _ = write!(result, "{item}, ");
        }
        _ = write!(result, "{last} or {last_expected}");
        result.into()
    } else {
        last_expected
    }
}

fn to_diagnostics<A: Display>(errors: ParseErrors<A>) -> Vec<Diagnostic> {
    errors.into_iter().map(to_diagnostic).collect()
}

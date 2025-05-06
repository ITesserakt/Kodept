use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::scope::Scoped;
use crate::symbol::table::SymbolTable;
use crate::symbol::Symbol;
use crate::wrapper::InteractionExt;
use crate::{done, fail, Ctx, Interaction, Result};
use bevy_ecs::prelude::{ChildOf, Children, Entity, IntoScheduleConfigs, Query};
use bevy_ecs::query::With;
use kodept_ast::properties::{Name, SourceSpan};
use kodept_ast::Str;
use kodept_ast_nodes::term::{Identifier, Ref, ReferenceContext};
use kodept_core::code_point::Span;
use kodept_report::message::{Diagnostic, Label, Severity};
use kodept_report::traits::IntoSpannedReportMessage;
use std::collections::VecDeque;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

pub struct ReferenceResolver;

#[derive(Debug)]
struct Path {
    context: ReferenceContext,
    ident: String,
}

#[derive(Debug)]
enum Error {
    UnknownReference {
        path: Path,
        location: Span,
    },
    #[allow(dead_code)]
    UnknownPath {
        path: Path,
        failed_segment: Option<Str>,
        location: Span,
    },
    Unsupported {
        location: Span,
    },
}

impl From<&Ref> for Path {
    fn from(value: &Ref) -> Self {
        Self {
            context: value.context.clone(),
            ident: value.ident.name().to_string(),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.context.global {
            write!(f, "::")?;
        }
        for item in &self.context.items {
            write!(f, "{}::", item)?;
        }
        write!(f, "{}", self.ident)
    }
}

impl IntoSpannedReportMessage for Error {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        match self {
            Error::UnknownPath {
                path,
                failed_segment: Some(segment),
                location,
            } => Diagnostic::new(Severity::Error)
                .with_message(format!("Cannot find `{}`", segment))
                .with_note(format!("While searching for symbol `{}`", path))
                .with_label(Label::primary("path not found", location)),
            Error::UnknownReference { path, location }
            | Error::UnknownPath { path, location, .. } => Diagnostic::new(Severity::Error)
                .with_message(format!("Cannot find symbol `{}`", path))
                .with_label(Label::primary("symbol not found", location)),
            Error::Unsupported { location } => Diagnostic::new(Severity::Error)
                .with_message("Non-global paths with non-trivial context is unsupported for now")
                .with_label(Label::primary("unsupported path", location)),
        }
    }
}

impl Interaction for ReferenceResolver {
    type Error = Infallible;

    fn install(ctx: &mut Ctx) {
        ctx.register(
            Self::wrap_system(Self::system)
                .run_if(|query: Query<(), With<SymbolTable>>| !query.is_empty()),
        )
    }
}

type ScopeQuery<'q> = (
    &'q Scope,
    Option<&'q SymbolTable>,
    Option<&'q ChildOf>,
    Option<&'q Children>,
    Option<&'q Name>,
);
type NodeQuery<'q> = (Entity, &'q Scoped, &'q Ref, &'q SourceSpan);

impl ReferenceResolver {
    fn search_symbol<'a>(node: &Ref, table: &'a SymbolTable) -> Option<&'a Symbol> {
        match &node.ident {
            Identifier::TypeReference { name } => table.get_type(Name::new(name.clone())),
            Identifier::Reference { name } => table.get_value(Name::new(name.clone())).ok(),
        }
    }

    fn resolve_ref_without_context(node: NodeQuery, scopes: &Query<ScopeQuery>) -> Result<Error> {
        let mut current_scope_id = node.1 .0;
        loop {
            let (_, symbols, parent, ..) = scopes.get(current_scope_id).unwrap();
            let symbol = symbols.and_then(|table| Self::search_symbol(node.2, table));
            if symbol.is_some() {
                return done();
            }
            if let Some(parent) = parent {
                current_scope_id = parent.0;
                continue;
            } else {
                return fail(Error::UnknownReference {
                    path: node.2.into(),
                    location: node.3 .0,
                });
            }
        }
    }

    fn resolve_ref_with_global_context(
        node: NodeQuery,
        scopes: &Query<ScopeQuery>,
    ) -> Result<Error> {
        enum ControlFlow {
            Value(Entity),
            Layer,
        }

        let (.., children, _) = scopes.iter().find(|it| it.2.is_none()).unwrap();
        let mut queue = children
            .into_iter()
            .flatten()
            .map(|it| ControlFlow::Value(*it))
            .chain(Some(ControlFlow::Layer))
            .collect::<VecDeque<_>>();

        let mut context_path_iter = node.2.context.items.iter().peekable();
        while let Some(flow) = queue.pop_front() {
            match flow {
                ControlFlow::Value(id) => {
                    let (scope, symbols, _, children, name) = scopes.get(id).unwrap();
                    let Some(name) = name else { continue };
                    let Some(context_path) = context_path_iter.peek() else {
                        continue;
                    };
                    if *context_path != name.deref() {
                        continue;
                    };

                    if !scope.is_anonymous && context_path_iter.len() == 1 {
                        let symbol = symbols.and_then(|it| Self::search_symbol(node.2, it));
                        return if symbol.is_some() {
                            done()
                        } else {
                            fail(Error::UnknownReference {
                                path: node.2.into(),
                                location: node.3 .0,
                            })
                        };
                    }

                    for child in children.into_iter().flatten() {
                        queue.push_back(ControlFlow::Value(*child));
                    }
                }
                ControlFlow::Layer => {
                    if queue.is_empty() {
                        break;
                    }
                    queue.push_back(ControlFlow::Layer);
                    context_path_iter.next();
                }
            };
        }
        fail(Error::UnknownPath {
            path: node.2.into(),
            failed_segment: context_path_iter.peek().map(|it| (*it).clone()),
            location: node.3 .0,
        })
    }

    fn system(
        query: Query<(Entity, &Scoped, &Ref, &SourceSpan)>,
        mut reporter: Reporter,
    ) -> Result<Infallible> {
        for (_, _, _, span) in query.iter() {
            reporter.report_ad_hoc(|| {
                Diagnostic::new(Severity::Note).with_label(Label::primary("unresolved ref", span.0))
            })
        }
        done()
    }
}

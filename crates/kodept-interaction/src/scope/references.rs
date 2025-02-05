use bevy_ecs::schedule::IntoSystemConfigs;
use crate::prelude::Symbol;
use crate::report::Reporter;
use crate::scope::storage::Scope;
use crate::scope::symbol::SymbolTable;
use crate::scope::ScopeMapping;
use crate::wrapper::InteractionWrapper;
use crate::{done, fail, Interaction, Result, Skip};
use bevy_ecs::prelude::{Entity, Populated, Query, Res};
use bevy_hierarchy::Parent;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::Str;
use kodept_ast_nodes::term::{Identifier, Ref, ReferenceContext};
use kodept_core::code_point::CodePoint;
use kodept_report::error::report::{IntoSpannedReportMessage, Label, Severity};
use kodept_report::error::Diagnostic;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use bevy_ecs::query::With;

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
        location: CodePoint,
    },
    #[allow(dead_code)]
    UnknownPath {
        path: Path,
        failed_segment: Str,
        location: CodePoint,
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
            Error::UnknownReference { path, location } => Diagnostic::new(Severity::Error)
                .with_message(format!("Cannot find symbol `{}`", path))
                .with_label(Label::primary("symbol not found", location)),
            Error::UnknownPath {
                path,
                failed_segment,
                location,
            } => Diagnostic::new(Severity::Error)
                .with_message(format!("Cannot find `{}`", failed_segment))
                .with_note(format!("While searching for symbol `{}`", path))
                .with_label(Label::primary("path not found", location)),
        }
    }
}

impl Interaction for ReferenceResolver {
    type Error = Infallible;

    fn interaction() -> InteractionWrapper<Self::Error> {
        let config = InteractionWrapper::wrap(Self::system)
            .unwrap()
            .run_if(|query: Query<(), With<SymbolTable>>| !query.is_empty());
        InteractionWrapper::from_configs(config)
    }
}

impl ReferenceResolver {
    fn resolve_ref_without_context(
        id: Entity,
        node: &Ref,
        mapping: &ScopeMapping,
        scopes: &Query<(&Scope, Option<&SymbolTable>, Option<&Parent>)>,
        syntax: &SyntaxResolver,
    ) -> Result<Error> {
        fn search_symbol<'a>(node: &Ref, table: &'a SymbolTable) -> Option<&'a Symbol> {
            match &node.ident {
                Identifier::TypeReference { name } => table.get_type(name),
                Identifier::Reference { name } => table.get_value(name),
            }
        }

        let scope_id = mapping.enclosing_scope_id(id);
        let mut current_scope_id = scope_id;
        loop {
            let (scope, symbols, parent) = scopes.get(current_scope_id).unwrap();
            let symbol = symbols.and_then(|table| search_symbol(node, table));
            if symbol.is_some() && !scope.opaque {
                return done();
            } else if let Some(parent) = parent {
                current_scope_id = parent.get();
                continue;
            } else {
                return fail(Error::UnknownReference {
                    path: node.into(),
                    location: syntax.get_location(id),
                });
            }
        }
    }

    fn system(
        query: Query<(Entity, &Ref)>,
        scopes: Populated<(&Scope, Option<&SymbolTable>, Option<&Parent>)>,
        scopes_mapping: Res<ScopeMapping>,
        syntax: Res<SyntaxResolver>,
        reporter: Reporter,
    ) -> Result<Infallible> {
        for (entity, node) in query.into_iter() {
            if !node.context.global && node.context.items.is_empty() {
                match Self::resolve_ref_without_context(
                    entity,
                    node,
                    &scopes_mapping,
                    &scopes,
                    &syntax,
                ) {
                    Err(Skip::Failed(e)) => reporter.report(e),
                    _ => {}
                }
            }
        }
        done()
    }
}

use crate::path::Path;
use crate::scope::ScopeSearcher;
use crate::symbol::SymbolKind;
use kodept_ast::graph::{Identifiable, SyntaxTree};
use kodept_ast::interning::SharedStr;
use kodept_ast::utils::Skip;
use kodept_ast::utils::Skip::Skipped;
use kodept_ast::{Identifier, Ref};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use kodept_macros::context::Context;
use kodept_macros::error::report::{IntoSpannedReportMessage, Label, Severity};
use kodept_macros::error::Diagnostic;
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::{Macro, MacroExt};
use SymbolKind::*;

pub struct RefResolver<'a> {
    scope_searcher: ScopeSearcher<'a>,
}

#[derive(Debug)]
pub enum RefResolverError {
    UnknownReference {
        path: Path,
        location: CodePoint,
    },
    UnknownPath {
        path: Path,
        failed_segment: SharedStr,
        location: CodePoint,
    },
}

impl IntoSpannedReportMessage for RefResolverError {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        let diag = Diagnostic::new(Severity::Error);
        let diag = match &self {
            RefResolverError::UnknownReference { path, .. } => {
                diag.with_message(format!("Cannot find symbol `{}`", path))
            }
            RefResolverError::UnknownPath {
                path,
                failed_segment,
                ..
            } => diag
                .with_message(format!("Cannot find `{}`", failed_segment))
                .with_note(format!("While searching for symbol `{}`", path)),
        };
        diag.with_label(Label::primary(
            "here",
            match self {
                RefResolverError::UnknownReference { location, .. } => location,
                RefResolverError::UnknownPath { location, .. } => location,
            },
        ))
    }
}

impl<'a> RefResolver<'a> {
    pub fn new(scope_searcher: ScopeSearcher<'a>) -> Self {
        Self { scope_searcher }
    }

    fn reference_kind_matcher(name: &Identifier) -> impl Fn(SymbolKind) -> bool {
        match name {
            Identifier::TypeReference { .. } => |k| matches!(k, Type),
            Identifier::Reference { .. } => |k| matches!(k, Parameter | Function | Variable),
        }
    }

    fn resolve_reference_without_context(&self, node: &Ref, ast: &SyntaxTree) -> bool {
        let id = node.get_id().widen();
        let enclosing_scope = self.scope_searcher.get_enclosing_scope(id, ast);

        self.scope_searcher
            .walk_bottom_up(enclosing_scope)
            .any(|it| {
                it.contains_symbol(node.ident.name(), Self::reference_kind_matcher(&node.ident))
            })
    }

    fn resolve_reference_with_context(&self, node: &Ref) -> Result<(), Option<SharedStr>> {
        match self.scope_searcher.matches(&node.context) {
            Ok(scope_for_context) => {
                if scope_for_context
                    .contains_symbol(node.ident.name(), Self::reference_kind_matcher(&node.ident))
                {
                    Ok(())
                } else {
                    Err(None)
                }
            }
            Err((_, missing_segment)) => Err(missing_segment.cloned()),
        }
    }
}

impl Macro for RefResolver<'_> {
    type Error = RefResolverError;
    type Node = Ref;
    type Ctx<'a> = Context<'a>;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Result<(), Skip<Self::Error>> {
        let id = guard.allow_last().ok_or(Skipped)?;
        let node = self.resolve(id, ctx);
        let path = Path {
            context: node.context.clone(),
            ident: node.ident.name().clone(),
        };

        if !node.context.global && node.context.items.is_empty() {
            if !self.resolve_reference_without_context(node, &ctx.ast) {
                Err(RefResolverError::UnknownReference {
                    path,
                    location: ctx.rlt.get_unknown(id).unwrap().location(),
                })?;
            }
            Ok(())
        } else {
            match self.resolve_reference_with_context(node) {
                Ok(_) => Ok(()),
                Err(Some(missing_segment)) => Err(RefResolverError::UnknownPath {
                    path,
                    failed_segment: missing_segment,
                    location: ctx.rlt.get_unknown(id).unwrap().location(),
                })?,
                Err(None) => Err(RefResolverError::UnknownReference {
                    path,
                    location: ctx.rlt.get_unknown(id).unwrap().location(),
                })?,
            }
        }
    }
}

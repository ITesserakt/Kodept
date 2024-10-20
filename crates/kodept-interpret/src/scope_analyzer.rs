use crate::scope::{ScopeBuilder, ScopePeelError, ScopeV2};
use crate::symbol::{SymbolKind, SymbolV2};
use kodept_ast::graph::node_props::Node;
use kodept_ast::graph::{AnyNode, AnyNodeId, Identifiable, SyntaxTree};
use kodept_ast::interning::SharedStr;
use kodept_ast::utils::Skip;
use kodept_ast::visit_side::VisitSide;
use kodept_ast::{
    AbstFnDecl, BodyFnDecl, EnumDecl, Exprs, ModDecl, NonTyParam, StructDecl, TyName, TyParam,
    VarDecl,
};
use kodept_core::code_point::CodePoint;
use kodept_core::structure::Located;
use kodept_macros::context::Context;
use kodept_macros::error::report::{IntoSpannedReportMessage, Label, Severity};
use kodept_macros::error::traits::SpannedError;
use kodept_macros::error::Diagnostic;
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::{Macro, MacroExt};

#[derive(Debug)]
pub struct DuplicatedSymbolErrorData {
    bound_name: SharedStr,
    previous_def_id: AnyNodeId,
}

#[derive(Debug)]
pub struct DuplicatedSymbolError {
    bound_name: SharedStr,
    current_def_location: CodePoint,
    previous_def_location: CodePoint,
}

pub struct ScopeAnalyzer {
    builder: ScopeBuilder,
}

impl IntoSpannedReportMessage for DuplicatedSymbolError {
    type Message = Diagnostic;

    fn into_message(self) -> Self::Message {
        Diagnostic::new(Severity::Error)
            .with_label(Label::primary("here", self.current_def_location))
            .with_label(Label::secondary("previous declaration", self.previous_def_location))
            .with_message(format!(
                "Element with name `{}` already defined",
                self.bound_name
            ))
    }
}

impl ScopeAnalyzer {
    pub fn new() -> ScopeAnalyzer {
        Self {
            builder: ScopeBuilder::new(),
        }
    }

    fn divide_by_scopes(&mut self, node: &AnyNode, side: VisitSide) -> Result<(), ScopePeelError> {
        // Optional name of a new scope, id it starts from and anon modifier
        let subdivision_meta = match node {
            // named scopes
            AnyNode::ModDecl(ModDecl { name, .. }) => Some((Some(name), None, false)),
            AnyNode::StructDecl(StructDecl { name, .. }) => Some((Some(name), None, false)),
            AnyNode::EnumDecl(EnumDecl { name, .. }) => Some((Some(name), None, false)),

            // Function form an anonymous scope, so variables inside is invisible
            AnyNode::AbstFnDecl(AbstFnDecl { name, .. }) => Some((Some(name), None, true)),
            // TODO: does it correct?
            AnyNode::BodyFnDecl(BodyFnDecl { name, .. }) => Some((Some(name), None, true)),

            // unnamed scopes (essentially anonymous)
            AnyNode::Lambda(_) => Some((None, None, true)),
            AnyNode::Exprs(Exprs { .. }) => Some((None, None, true)),
            AnyNode::IfExpr(_) => Some((None, None, true)),

            // do not divide
            AnyNode::FileDecl(_) => None,
            AnyNode::TyParam(_) => None,
            AnyNode::NonTyParam(_) => None,
            AnyNode::TyName(_) => None,
            AnyNode::VarDecl(_) => None,
            AnyNode::InitVar(_) => None,
            AnyNode::Appl(_) => None,
            AnyNode::Ref(_) => None,
            AnyNode::Acc(_) => None,
            AnyNode::NumLit(_) => None,
            AnyNode::CharLit(_) => None,
            AnyNode::StrLit(_) => None,
            AnyNode::TupleLit(_) => None,
            AnyNode::ElifExpr(_) => None,
            AnyNode::ElseExpr(_) => None,
            AnyNode::BinExpr(_) => None,
            AnyNode::UnExpr(_) => None,
            AnyNode::ProdTy(_) => None,
            // do not put `_` here, process each new case individually
        };

        if let Some((name, override_start, is_anonymous)) = subdivision_meta {
            let start = override_start.unwrap_or(node.get_id());
            match side {
                VisitSide::Entering | VisitSide::Leaf => {
                    self.builder.push_scope(start, name, Some(is_anonymous));
                    return Ok(());
                }
                VisitSide::Exiting => {
                    // It shouldn't be possible to go outside of root, because FileDecl (root node) is not used above.
                    // However, in other configurations of the AST, this contract may not hold
                    self.builder.peel_scope()?;
                }
            }
        }
        Ok(())
    }

    pub fn into_inner(self) -> ScopeBuilder {
        self.builder
    }
}

fn extract_symbols(
    destination_scope: &mut ScopeV2,
    node: &AnyNode,
    ast: &SyntaxTree,
) -> Result<(), DuplicatedSymbolErrorData> {
    let id = node.get_id();

    let old_symbol =
        match node {
            AnyNode::StructDecl(StructDecl { name, .. }) => {
                destination_scope.insert_symbol(SymbolV2::new(id, name.clone(), SymbolKind::Type))
            }
            AnyNode::EnumDecl(EnumDecl { name, .. }) => {
                destination_scope.insert_symbol(SymbolV2::new(id, name.clone(), SymbolKind::Type))
            }

            AnyNode::AbstFnDecl(AbstFnDecl { name, .. }) => destination_scope
                .insert_symbol(SymbolV2::new(id, name.clone(), SymbolKind::Function)),
            AnyNode::BodyFnDecl(BodyFnDecl { name, .. }) => destination_scope
                .insert_symbol(SymbolV2::new(id, name.clone(), SymbolKind::Function)),

            AnyNode::VarDecl(VarDecl { name, .. }) => destination_scope
                .insert_symbol(SymbolV2::new(id, name.clone(), SymbolKind::Variable)),

            AnyNode::TyName(ref ty @ TyName { name, .. }) => {
                // enum struct $name { $ty_name1, $ty_name2 }
                // insertion happens in the inner enum scope
                if ty.parent_is::<EnumDecl>(ast) {
                    destination_scope.insert_symbol(SymbolV2::new(
                        id,
                        name.clone(),
                        SymbolKind::Parameter,
                    ))
                } else {
                    None
                }
            }
            AnyNode::TyParam(TyParam { name, .. })
            | AnyNode::NonTyParam(NonTyParam { name, .. }) => destination_scope
                .insert_symbol(SymbolV2::new(id, name.clone(), SymbolKind::Parameter)),

            // Again, handle each new case manually
            AnyNode::FileDecl(_) => None,
            AnyNode::ModDecl(_) => None,
            AnyNode::InitVar(_) => None,
            AnyNode::Exprs(_) => None,
            AnyNode::Appl(_) => None,
            AnyNode::Lambda(_) => None,
            AnyNode::Ref(_) => None,
            AnyNode::Acc(_) => None,
            AnyNode::NumLit(_) => None,
            AnyNode::CharLit(_) => None,
            AnyNode::StrLit(_) => None,
            AnyNode::TupleLit(_) => None,
            AnyNode::IfExpr(_) => None,
            AnyNode::ElifExpr(_) => None,
            AnyNode::ElseExpr(_) => None,
            AnyNode::BinExpr(_) => None,
            AnyNode::UnExpr(_) => None,
            AnyNode::ProdTy(_) => None,
        };

    if let Some(old_symbol) = old_symbol {
        Err(DuplicatedSymbolErrorData {
            bound_name: old_symbol.ident,
            previous_def_id: old_symbol.ast_node,
        })
    } else {
        Ok(())
    }
}

impl Macro for ScopeAnalyzer {
    type Error = DuplicatedSymbolError;
    type Node = AnyNode;
    type Ctx<'a> = Context<'a>;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Result<(), Skip<Self::Error>> {
        let (id, side) = guard.allow_all();
        let node = self.resolve(id, ctx);

        if let Err(e) = self.divide_by_scopes(node, side) {
            ctx.report(SpannedError::for_node(e, id, &ctx.rlt).with_severity(Severity::Bug));
        }

        // Ensures that we're using enclosing scope for nodes
        if matches!(side, VisitSide::Entering) {
            return Ok(());
        }

        let scope = self.builder.current_scope_mut();
        extract_symbols(scope, node, &ctx.ast).map_err(|e| DuplicatedSymbolError {
            bound_name: e.bound_name,
            current_def_location: ctx.rlt.get_unknown(id).unwrap().location(),
            previous_def_location: ctx.rlt.get_unknown(e.previous_def_id).unwrap().location(),
        })?;

        Ok(())
    }
}

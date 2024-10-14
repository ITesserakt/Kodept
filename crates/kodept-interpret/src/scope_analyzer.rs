use crate::scope::{ScopeBuilder, ScopePeelError, ScopeV2};
use kodept_ast::graph::node_props::SubEnum;
use kodept_ast::graph::{AnyNode, Identifiable};
use kodept_ast::visit_side::VisitSide;
use kodept_ast::{AbstFnDecl, BodyFnDecl, EnumDecl, Exprs, FileDecl, ModDecl, StructDecl};
use kodept_macros::context::Context;
use kodept_macros::error::report::Severity;
use kodept_macros::error::traits::SpannedError;
use kodept_macros::execution::Execution;
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::{Macro, MacroExt};
use std::convert::Infallible;

pub struct ScopeAnalyzer {
    builder: ScopeBuilder,
}

impl ScopeAnalyzer {
    pub fn new() -> ScopeAnalyzer {
        Self {
            builder: ScopeBuilder::new(),
        }
    }

    fn divide_by_scopes(&mut self, node: &AnyNode, side: VisitSide) -> Result<(), ScopePeelError> {
        // Optional name of a new scope and id it starts from
        let subdivision_meta = match node {
            // named scopes
            AnyNode::ModDecl(ModDecl { name, .. }) => Some((Some(name), None)),
            AnyNode::StructDecl(StructDecl { name, .. }) => Some((Some(name), None)),
            AnyNode::EnumDecl(EnumDecl { name, .. }) => Some((Some(name), None)),
            AnyNode::AbstFnDecl(AbstFnDecl { name, .. }) => Some((Some(name), None)),
            // TODO: does it correct?
            AnyNode::BodyFnDecl(BodyFnDecl { name, .. }) => Some((Some(name), None)),

            // unnamed scopes
            AnyNode::Lambda(_) => Some((None, None)),
            AnyNode::Exprs(Exprs { .. }) => Some((None, None)),
            AnyNode::IfExpr(_) => Some((None, None)),

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

        if let Some((name, override_start)) = subdivision_meta {
            let start = override_start.unwrap_or(node.get_id());
            match side {
                VisitSide::Entering | VisitSide::Leaf => {
                    let scope = self.builder.push_scope(start);
                    scope.name = name.cloned();
                    return Ok(());
                }
                // TODO: replace with if_let_guard when it'll become stable
                VisitSide::Exiting => {
                    // It shouldn't be possible to go outside of root, because FileDecl (root node) is not used above.
                    // However, in other configurations of the AST, this contract may not hold
                    self.builder.peel_scope()?;
                }
            }
        }
        Ok(())
    }
}

fn extract_symbols(destination_scope: &mut ScopeV2, node: &AnyNode) {
    let id = node.get_id();

    match node {
        AnyNode::FileDecl(_) => {}
        AnyNode::ModDecl(_) => {}
        AnyNode::StructDecl(StructDecl { name, .. }) => {
            // destination_scope.insert_symbol(SymbolV2::new(id, ))
        }
        AnyNode::EnumDecl(_) => {}
        AnyNode::TyParam(_) => {}
        AnyNode::NonTyParam(_) => {}
        AnyNode::TyName(_) => {}
        AnyNode::VarDecl(_) => {}
        AnyNode::InitVar(_) => {}
        AnyNode::BodyFnDecl(_) => {}
        AnyNode::Exprs(_) => {}
        AnyNode::Appl(_) => {}
        AnyNode::Lambda(_) => {}
        AnyNode::Ref(_) => {}
        AnyNode::Acc(_) => {}
        AnyNode::NumLit(_) => {}
        AnyNode::CharLit(_) => {}
        AnyNode::StrLit(_) => {}
        AnyNode::TupleLit(_) => {}
        AnyNode::IfExpr(_) => {}
        AnyNode::ElifExpr(_) => {}
        AnyNode::ElseExpr(_) => {}
        AnyNode::BinExpr(_) => {}
        AnyNode::UnExpr(_) => {}
        AnyNode::AbstFnDecl(_) => {}
        AnyNode::ProdTy(_) => {}
    }
}

impl Macro for ScopeAnalyzer {
    type Error = Infallible;
    type Node = AnyNode;
    type Ctx<'a> = Context<'a>;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Execution<Self::Error> {
        let (id, side) = guard.allow_all();
        let node = self.resolve(id, ctx);

        if let Err(e) = self.divide_by_scopes(node, side) {
            ctx.report(SpannedError::for_node(e, id, &ctx.rlt).with_severity(Severity::Bug));
        }
        let node = ctx.ast.get(id)?;
        
        if FileDecl::contains(node) {
            // divide_by_scopes should correctly peel all the scopes to the root one
            assert_eq!(self.builder.root_scope(), self.builder.current_scope());
        }

        if matches!(side, VisitSide::Entering) {
            return Execution::Completed(());
        }

        let scope = self.builder.current_scope_mut();
        dbg!(&scope);

        extract_symbols(scope, node);

        Execution::Completed(())
    }
}

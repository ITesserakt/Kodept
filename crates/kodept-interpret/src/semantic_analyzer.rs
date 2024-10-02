use crate::scope::{ScopeError, ScopeTree};
use kodept_ast::graph::{AnyNode};
use kodept_ast::traits::Identifiable;
use kodept_ast::visit_side::VisitSide;
use kodept_ast::{
    AbstFnDecl, BodyFnDecl, EnumDecl, ModDecl, NonTyParam, StructDecl, TyName, TyParam, VarDecl,
};
use kodept_inference::r#type::MonomorphicType::Constant;
use kodept_macros::context::Context;
use kodept_macros::error::traits::SpannedError;
use kodept_macros::execution::Execution;
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::{Macro, MacroExt};

pub struct ScopeAnalyzer(ScopeTree);

impl Default for ScopeAnalyzer {
    fn default() -> Self {
        Self(ScopeTree::new())
    }
}

impl ScopeAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> ScopeTree {
        self.0
    }

    fn divide_by_scopes(&mut self, node: &AnyNode, side: VisitSide) -> Result<(), ScopeError> {
        let divide = match node {
            AnyNode::ModDecl(ModDecl { name, .. }) => Some(Some(name)),
            AnyNode::StructDecl(StructDecl { name, .. }) => Some(Some(name)),
            AnyNode::EnumDecl(EnumDecl { name, .. }) => Some(Some(name)),
            AnyNode::AbstFnDecl(AbstFnDecl { name, .. }) => Some(Some(name)),
            AnyNode::BodyFnDecl(BodyFnDecl { name, .. }) => Some(Some(name)),
            AnyNode::FileDecl(_) => Some(None),
            AnyNode::Exprs(_) => Some(None),
            AnyNode::Lambda(_) => Some(None),
            AnyNode::IfExpr(_) => Some(None),
            AnyNode::ElifExpr(_) => Some(None),
            AnyNode::ElseExpr(_) => Some(None),
            _ => None,
        };

        if let Some(name) = divide {
            if side == VisitSide::Entering {
                self.0.push_scope(node, name)
            }
            if side == VisitSide::Exiting {
                self.0.pop_scope()?
            }
        }
        Ok(())
    }
}

impl Macro for ScopeAnalyzer {
    type Error = SpannedError<ScopeError>;
    type Node = AnyNode;
    type Ctx<'a> = Context<'a>;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut Self::Ctx<'_>,
    ) -> Execution<Self::Error> {
        let (id, side) = guard.allow_all();
        let node = self.resolve(id, ctx);

        self.divide_by_scopes(node, side)
            .map_err(|e| SpannedError::for_node(e, id, &ctx.rlt))?;

        if !matches!(side, VisitSide::Exiting | VisitSide::Leaf) {
            return Execution::Skipped;
        }

        let Ok(scope) = self.0.current_mut() else {
            return Execution::Skipped;
        };
        match node {
            AnyNode::StructDecl(StructDecl { name, .. }) => {
                scope.insert_type(name, Constant(name.to_string()))
            }
            AnyNode::TyParam(TyParam { name, .. }) => scope.insert_var(id, name),
            AnyNode::NonTyParam(NonTyParam { name, .. }) => scope.insert_var(id, name),
            AnyNode::TyName(TyName { name, .. }) => {
                if let Some(AnyNode::EnumDecl(_)) = ctx.ast.parent_of(id) {
                    scope.insert_type(name, Constant(name.to_string()))
                } else {
                    Ok(())
                }
            }
            AnyNode::EnumDecl(EnumDecl { name, .. }) => {
                scope.insert_type(name, Constant(name.to_string()))
            }
            AnyNode::VarDecl(VarDecl { name, .. }) => scope.insert_var(node.get_id(), name),
            AnyNode::BodyFnDecl(BodyFnDecl { name, .. }) => scope.insert_var(node.get_id(), name),
            AnyNode::AbstFnDecl(AbstFnDecl { name, .. }) => scope.insert_var(node.get_id(), name),
            _ => Ok(()),
        }
        .map_err(|e| SpannedError::for_node(e, id, &ctx.rlt))?;

        Execution::Completed(())
    }
}

use tracing::debug;

use crate::scope::{ScopeError, ScopeTree};
use kodept_ast::graph::{AnyNode, ChangeSet};
use kodept_ast::traits::Identifiable;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::VisitSide;
use kodept_ast::{
    AbstFnDecl, BodyFnDecl, EnumDecl, ModDecl, NonTyParam, StructDecl, TyName, TyParam, VarDecl,
};
use kodept_inference::r#type::MonomorphicType::Constant;
use kodept_macros::context::{Context, SyntaxProvider};
use kodept_macros::visit_guard::VisitGuard;
use kodept_macros::Macro;

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

impl<C: SyntaxProvider> Macro<C> for ScopeAnalyzer {
    type Error = ScopeError;
    type Node = AnyNode;

    fn apply(
        &mut self,
        guard: VisitGuard<Self::Node>,
        ctx: &mut impl Context<C>,
    ) -> Execution<Self::Error, ChangeSet> {
        let (id, side) = guard.allow_all();
        let node = self.resolve(id, ctx);

        if side == VisitSide::Exiting {
            debug!("{:#?}", self.0);
        }

        self.divide_by_scopes(node, side)?;

        if !matches!(side, VisitSide::Exiting | VisitSide::Leaf) {
            return Execution::Skipped;
        }

        let Ok(scope) = self.0.current_mut() else {
            return Execution::Skipped;
        };
        match node {
            AnyNode::StructDecl(StructDecl { name, .. }) => {
                scope.insert_type(name, Constant(name.clone()))?;
            }
            AnyNode::TyParam(TyParam { name, .. }) => {
                scope.insert_var(id, name)?;
            }
            AnyNode::NonTyParam(NonTyParam { name, .. }) => {
                scope.insert_var(id, name)?;
            }
            AnyNode::TyName(TyName { name, .. }) => {
                if let Some(AnyNode::EnumDecl(_)) = ctx.parent_of(id) {
                    scope.insert_type(name, Constant(name.clone()))?;
                }
            }
            AnyNode::EnumDecl(EnumDecl { name, .. }) => {
                scope.insert_type(name, Constant(name.clone()))?;
            }
            AnyNode::VarDecl(VarDecl { name, .. }) => scope.insert_var(node.get_id(), name)?,
            AnyNode::BodyFnDecl(BodyFnDecl { name, .. }) => {
                scope.insert_var(node.get_id(), name)?;
            }
            AnyNode::AbstFnDecl(AbstFnDecl { name, .. }) => {
                scope.insert_var(node.get_id(), name)?
            }
            _ => {}
        }

        Execution::Completed(ChangeSet::new())
    }
}

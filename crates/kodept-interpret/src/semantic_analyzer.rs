use tracing::debug;

use kodept_ast::graph::{ChangeSet, GenericASTNode};
use kodept_ast::traits::Identifiable;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_ast::{
    AbstractFunctionDeclaration, BodiedFunctionDeclaration, EnumDeclaration, ModuleDeclaration,
    StructDeclaration, TypeName, TypedParameter, UntypedParameter, Variable,
};
use kodept_inference::r#type::MonomorphicType::Constant;
use kodept_macros::traits::Context;
use kodept_macros::Macro;

use crate::scope::{ScopeError, ScopeTree};

pub struct ScopeAnalyzer(ScopeTree, usize);

impl Default for ScopeAnalyzer {
    fn default() -> Self {
        Self(ScopeTree::new(), 0)
    }
}

impl ScopeAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_inner(self) -> ScopeTree {
        self.0
    }

    fn divide_by_scopes(
        &mut self,
        node: &GenericASTNode,
        side: VisitSide,
    ) -> Result<(), ScopeError> {
        let divide = match node {
            GenericASTNode::Module(ModuleDeclaration { name, .. }) => Some(Some(name)),
            GenericASTNode::Struct(StructDeclaration { name, .. }) => Some(Some(name)),
            GenericASTNode::Enum(EnumDeclaration { name, .. }) => Some(Some(name)),
            GenericASTNode::AbstractFunction(AbstractFunctionDeclaration { name, .. }) => {
                Some(Some(name))
            }
            GenericASTNode::BodiedFunction(BodiedFunctionDeclaration { name, .. }) => {
                Some(Some(name))
            }
            GenericASTNode::File(_) => Some(None),
            GenericASTNode::ExpressionBlock(_) => Some(None),
            GenericASTNode::Lambda(_) => Some(None),
            GenericASTNode::If(_) => Some(None),
            GenericASTNode::Elif(_) => Some(None),
            GenericASTNode::Else(_) => Some(None),
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
    type Error = ScopeError;
    type Node = GenericASTNode;

    fn transform(
        &mut self,
        guard: VisitGuard<Self::Node>,
        context: &mut impl Context,
    ) -> Execution<Self::Error, ChangeSet> {
        let (node, side) = guard.allow_all();

        if side == VisitSide::Exiting {
            debug!("{:#?}", self.0);
        }

        self.divide_by_scopes(&node, side)?;

        if !matches!(side, VisitSide::Exiting | VisitSide::Leaf) {
            return Execution::Skipped;
        }

        let Ok(scope) = self.0.current_mut() else {
            return Execution::Skipped;
        };
        let Some(tree) = context.tree().upgrade() else {
            return Execution::Skipped;
        };
        match &*node {
            GenericASTNode::Struct(StructDeclaration { name, .. }) => {
                scope.insert_type(name, Constant(self.1).into())?;
                self.1 += 1;
            }
            GenericASTNode::TypedParameter(TypedParameter { name, .. }) => {
                scope.insert_var(name)?;
            }
            GenericASTNode::UntypedParameter(UntypedParameter { name, .. }) => {
                scope.insert_var(name)?;
            }
            GenericASTNode::TypeName(TypeName { name, .. }) => {
                if let Some(GenericASTNode::Enum(_)) = tree.parent_of(node.get_id(), node.token()) {
                    scope.insert_type(name, Constant(self.1).into())?;
                    self.1 += 1;
                }
            }
            GenericASTNode::Enum(EnumDeclaration { name, .. }) => {
                scope.insert_type(name, Constant(self.1).into())?;
                self.1 += 1;
            }
            GenericASTNode::Variable(Variable { name, .. }) => scope.insert_var(name)?,
            GenericASTNode::BodiedFunction(BodiedFunctionDeclaration { name, .. }) => {
                scope.insert_var(name)?;
            }
            GenericASTNode::AbstractFunction(AbstractFunctionDeclaration { name, .. }) => {
                scope.insert_var(name)?
            }
            _ => {}
        }

        Execution::Completed(ChangeSet::new())
    }
}

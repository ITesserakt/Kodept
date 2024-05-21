use tracing::debug;

use kodept_ast::{
    AbstractFunctionDeclaration, BodiedFunctionDeclaration, EnumDeclaration, ModuleDeclaration,
    StructDeclaration, TypedParameter, TypeName, UntypedParameter, Variable,
};
use kodept_ast::graph::{ChangeSet, AnyNode};
use kodept_ast::traits::Identifiable;
use kodept_ast::utils::Execution;
use kodept_ast::visit_side::{VisitGuard, VisitSide};
use kodept_inference::r#type::MonomorphicType::Constant;
use kodept_macros::Macro;
use kodept_macros::traits::Context;

use crate::scope::{ScopeError, ScopeTree};

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

    fn divide_by_scopes(
        &mut self,
        node: &AnyNode,
        side: VisitSide,
    ) -> Result<(), ScopeError> {
        let divide = match node {
            AnyNode::Module(ModuleDeclaration { name, .. }) => Some(Some(name)),
            AnyNode::Struct(StructDeclaration { name, .. }) => Some(Some(name)),
            AnyNode::Enum(EnumDeclaration { name, .. }) => Some(Some(name)),
            AnyNode::AbstractFunction(AbstractFunctionDeclaration { name, .. }) => {
                Some(Some(name))
            }
            AnyNode::BodiedFunction(BodiedFunctionDeclaration { name, .. }) => {
                Some(Some(name))
            }
            AnyNode::File(_) => Some(None),
            AnyNode::ExpressionBlock(_) => Some(None),
            AnyNode::Lambda(_) => Some(None),
            AnyNode::If(_) => Some(None),
            AnyNode::Elif(_) => Some(None),
            AnyNode::Else(_) => Some(None),
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
    type Node = AnyNode;

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
            AnyNode::Struct(StructDeclaration { name, .. }) => {
                scope.insert_type(name, Constant(name.clone()).into())?;
            }
            AnyNode::TypedParameter(TypedParameter { name, .. }) => {
                scope.insert_var(name)?;
            }
            AnyNode::UntypedParameter(UntypedParameter { name, .. }) => {
                scope.insert_var(name)?;
            }
            AnyNode::TypeName(TypeName { name, .. }) => {
                if let Some(AnyNode::Enum(_)) = tree.parent_of(node.get_id(), node.token()) {
                    scope.insert_type(name, Constant(name.clone()).into())?;
                }
            }
            AnyNode::Enum(EnumDeclaration { name, .. }) => {
                scope.insert_type(name, Constant(name.clone()).into())?;
            }
            AnyNode::Variable(Variable { name, .. }) => scope.insert_var(name)?,
            AnyNode::BodiedFunction(BodiedFunctionDeclaration { name, .. }) => {
                scope.insert_var(name)?;
            }
            AnyNode::AbstractFunction(AbstractFunctionDeclaration { name, .. }) => {
                scope.insert_var(name)?
            }
            _ => {}
        }

        Execution::Completed(ChangeSet::new())
    }
}

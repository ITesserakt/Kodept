use crate::scope::{ScopeError, ScopeTree};
use derive_more::{From, Into};
use kodept_ast::graph::{GenericASTNode, GhostToken, NodeUnion, SyntaxTree};
use kodept_ast::Identifier::TypeReference;
use kodept_ast::{
    wrapper, AbstractFunctionDeclaration, Application, BodiedFunctionDeclaration, ExpressionBlock,
    Identifier, IfExpression, InitializedVariable, Lambda, Literal, Reference, TypedParameter,
    Variable,
};
use kodept_inference::assumption::Assumptions;
use kodept_inference::language::{var, Var};

wrapper! {
    #[derive(From, Into, PartialEq, Debug)]
    wrapper TypeDerivableNode {
        function(BodiedFunctionDeclaration) = GenericASTNode::BodiedFunction(x) => Some(x),
        expression_block(ExpressionBlock) = GenericASTNode::ExpressionBlock(x) => Some(x),
        init_var(InitializedVariable) = GenericASTNode::InitializedVariable(x) => Some(x),
        lambda(Lambda) = GenericASTNode::Lambda(x) => Some(x),
        application(Application) = GenericASTNode::Application(x) => Some(x),
        if_expr(IfExpression) = GenericASTNode::If(x) => Some(x),
        reference(Reference) = GenericASTNode::Reference(x@Reference{ ident: Identifier::Reference { .. }, .. }) => Some(x),

        literal(Literal) = x if Literal::contains(x) => x.try_into().ok(),
    }
}

wrapper! {
    #[derive(From, Into)]
    wrapper TypeRestrictedNode {
        typed_parameter(TypedParameter) = GenericASTNode::TypedParameter(x) => Some(x),
        function(AbstractFunctionDeclaration) = GenericASTNode::AbstractFunction(x) => Some(x),
        variable(Variable) = GenericASTNode::Variable(x) => Some(x),
        reference(Reference) = GenericASTNode::Reference(x@Reference { ident: TypeReference { .. }, .. }) => Some(x),

        // Literals are not suitable here because they don't have name
    }
}

impl TypeRestrictedNode {
    pub fn var(&self) -> Var {
        if let Some(node) = self.as_typed_parameter() {
            var(&node.name)
        } else if let Some(Reference {
            ident: TypeReference { name },
            ..
        }) = self.as_reference()
        {
            var(name)
        } else if let Some(node) = self.as_function() {
            var(&node.name)
        } else if let Some(node) = self.as_variable() {
            var(&node.name)
        } else {
            unreachable!()
        }
    }

    pub fn type_of(
        &self,
        ast: &SyntaxTree,
        token: &GhostToken,
        scopes: &ScopeTree,
    ) -> Result<Assumptions, ScopeError> {
        todo!()
    }
}

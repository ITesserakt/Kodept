use derive_more::{From, TryInto};
use kodept_ast::*;

type Ref<'a, T> = &'a T;
type Mut<'a, T> = &'a mut T;
type Exact<T> = T;

macro_rules! make_ast_node_adapter {
    (ref $life:lifetime) => {
        kodept_ast::make_ast_node_adaptor!(ASTNode, lifetimes: [$life], Ref, configs: [
            derive(TryInto, From, Copy, Clone, Debug)
        ]);
    };
    (mut $life:lifetime) => {
        kodept_ast::make_ast_node_adaptor!(ASTNodeMut, lifetimes: [$life], Mut, configs: [
            derive(TryInto, From, Debug)
        ]);
    };
    () => {
        kodept_ast::make_ast_node_adaptor!(AnyASTNode, lifetimes: [], Exact, configs: [
            derive(TryInto, From, Debug)
        ]);
    }
}

make_ast_node_adapter!(ref 'a);
make_ast_node_adapter!(mut 'a);
make_ast_node_adapter!();

impl<'a> From<ASTNodeMut<'a>> for ASTNode<'a> {
    #[inline(always)]
    fn from(value: ASTNodeMut<'a>) -> Self {
        match value {
            ASTNodeMut::File(x) => ASTNode::File(x),
            ASTNodeMut::Module(x) => ASTNode::Module(x),
            ASTNodeMut::Struct(x) => ASTNode::Struct(x),
            ASTNodeMut::Enum(x) => ASTNode::Enum(x),
            ASTNodeMut::Type(x) => ASTNode::Type(x),
            ASTNodeMut::TypedParameter(x) => ASTNode::TypedParameter(x),
            ASTNodeMut::TopLevel(x) => ASTNode::TopLevel(x),
            ASTNodeMut::TypeName(x) => ASTNode::TypeName(x),
            ASTNodeMut::Variable(x) => ASTNode::Variable(x),
            ASTNodeMut::InitializedVariable(x) => ASTNode::InitializedVariable(x),
            ASTNodeMut::BodiedFunction(x) => ASTNode::BodiedFunction(x),
            ASTNodeMut::Body(x) => ASTNode::Body(x),
            ASTNodeMut::BlockLevel(x) => ASTNode::BlockLevel(x),
            ASTNodeMut::ExpressionBlock(x) => ASTNode::ExpressionBlock(x),
            ASTNodeMut::Operation(x) => ASTNode::Operation(x),
            ASTNodeMut::Application(x) => ASTNode::Application(x),
            ASTNodeMut::Lambda(x) => ASTNode::Lambda(x),
            ASTNodeMut::Expression(x) => ASTNode::Expression(x),
            ASTNodeMut::Term(x) => ASTNode::Term(x),
            ASTNodeMut::Reference(x) => ASTNode::Reference(x),
            ASTNodeMut::Access(x) => ASTNode::Access(x),
            ASTNodeMut::Number(x) => ASTNode::Number(x),
            ASTNodeMut::Char(x) => ASTNode::Char(x),
            ASTNodeMut::String(x) => ASTNode::String(x),
            ASTNodeMut::Tuple(x) => ASTNode::Tuple(x),
            ASTNodeMut::Literal(x) => ASTNode::Literal(x),
            ASTNodeMut::CodeFlow(x) => ASTNode::CodeFlow(x),
            ASTNodeMut::If(x) => ASTNode::If(x),
            ASTNodeMut::Elif(x) => ASTNode::Elif(x),
            ASTNodeMut::Else(x) => ASTNode::Else(x),
            ASTNodeMut::Binary(x) => ASTNode::Binary(x),
            ASTNodeMut::Unary(x) => ASTNode::Unary(x),
            ASTNodeMut::AbstractFunction(x) => ASTNode::AbstractFunction(x),
            ASTNodeMut::ResolvedReference(x) => ASTNode::ResolvedReference(x),
            ASTNodeMut::ResolvedTypeReference(x) => ASTNode::ResolvedTypeReference(x),
            ASTNodeMut::ProdType(x) => ASTNode::ProdType(x),
            ASTNodeMut::SumType(x) => ASTNode::SumType(x),
            ASTNodeMut::Identifier(x) => ASTNode::Identifier(x),
            ASTNodeMut::UntypedParameter(x) => ASTNode::UntypedParameter(x),
        }
    }
}

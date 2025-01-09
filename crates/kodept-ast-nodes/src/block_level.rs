use crate::properties::{BlockLevel, Type};
use crate::types::{ProdTy, TyName};
use crate::Unit;
use kodept_ast::external::Component;
use kodept_ast::prelude::{Choose, CodeHolder, FromSyntax};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_ast::{derive_node, Str};
use kodept_ast::syntax_tree::children::{ChildrenDisjoint, HasChild};
use kodept_rlt::{BlockLevelNode, InitializedVariable, Variable};
use crate::code_flow::IfExpr;
use crate::expression::{BinExpr, App, Exprs, Lambda, UnExpr};
use crate::function::Func;
use crate::literal::{Literal, Tuple};
use crate::term::Ref;

#[derive(Debug, PartialEq)]
pub enum VariableKind {
    Immutable,
    Mutable
}

#[derive(Debug, PartialEq, Component)]
pub struct VarDecl {
    pub kind: VariableKind,
    pub name: Str,
}

#[derive(Debug, PartialEq, Component)]
pub struct InitVar;

derive_node!(VarDecl {
    relations = [
        optional TyName where tag = Type,
        optional ProdTy where tag = Type,
    ],
    properties = []
});
derive_node!(InitVar {
    relations = [
        child VarDecl,
    ],
    properties = []
});

impl FromSyntax for VarDecl {
    type Syntax = Variable;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let (kind, id, ty) = match node {
            Variable::Immutable { id, assigned_type, .. } => (VariableKind::Immutable, id, assigned_type),
            Variable::Mutable { id, assigned_type, .. } => (VariableKind::Mutable, id, assigned_type)
        };
        let name = source.get_chunk_located(id);
        ASTBuilder::new(pool, VarDecl { kind, name }).with_children(source, pool, |scope| {
            scope.maybe_choose(Unit, ty.as_ref().map(|it| [&it.1]))
        })
    }
}

impl FromSyntax for InitVar {
    type Syntax = InitializedVariable;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        ASTBuilder::new(pool, InitVar).with_children(source, pool, |scope| {
            scope.many([&node.variable])
        })
    }
}

impl<R> Choose<BlockLevelNode, R, BlockLevel> for Unit
where
    R: HasChild<InitVar, BlockLevel>,
    R: HasChild<Func, BlockLevel>,
    R: HasChild<Exprs, BlockLevel>,
    R: HasChild<App, BlockLevel>,
    R: HasChild<Lambda, BlockLevel>,
    R: HasChild<IfExpr, BlockLevel>,
    R: HasChild<BinExpr, BlockLevel>,
    R: HasChild<UnExpr, BlockLevel>,
    R: HasChild<Ref, BlockLevel>,
    R: HasChild<Literal, BlockLevel>,
    R: HasChild<Tuple, BlockLevel>
{
    #[inline(always)]
    fn branch<Source: CodeHolder>(node: &BlockLevelNode) -> ChildrenDisjoint<R, Source, BlockLevel> {
        match node {
            BlockLevelNode::InitVar(x) => ChildrenDisjoint::new::<InitVar>(x),
            BlockLevelNode::Function(x) => ChildrenDisjoint::new::<Func>(x),
            BlockLevelNode::Operation(x) => Unit::branch(x),
            BlockLevelNode::Block(x) => ChildrenDisjoint::new::<Exprs>(x)
        }
    }
}

use crate::function::Func;
use crate::types::{TyName, TyParam};
use kodept_ast::external::Component;
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_ast::{derive_node, Str};
use kodept_rlt::{Enum, Struct};

#[derive(Debug, PartialEq)]
pub enum EnumKind {
    Stack,
    Heap,
}

#[derive(Debug, PartialEq, Component)]
pub struct EnumDecl {
    pub kind: EnumKind,
    pub name: Str,
}

#[derive(Debug, PartialEq, Component)]
pub struct StructDecl {
    pub name: Str,
}

derive_node!(EnumDecl {
    relations = [children TyName,],
    properties = []
});

derive_node!(StructDecl {
    relations = [
        children TyParam,
        children Func,
    ],
    properties = []
});

impl FromSyntax for EnumDecl {
    type Syntax = Enum;

    fn from_syntax(
        node: &Self::Syntax,
        source: impl CodeHolder,
        builder: &Pool,
    ) -> ASTBuilder<Self> {
        let (kind, id, rest) = match node {
            Enum::Stack { id, contents, .. } => (EnumKind::Stack, id, contents),
            Enum::Heap { id, contents, .. } => (EnumKind::Heap, id, contents),
        };
        let name = source.get_chunk_located(id);
        ASTBuilder::new(builder, EnumDecl { kind, name }).with_children(source, builder, |scope| {
            scope.maybe_many(rest.as_ref().map(|it| it.inner.as_ref()))
        })
    }
}

impl FromSyntax for StructDecl {
    type Syntax = Struct;

    fn from_syntax(node: &Struct, source: impl CodeHolder, builder: &Pool) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(builder, StructDecl { name }).with_children(source, builder, |scope| {
            scope.maybe_many::<TyParam, _>(node.parameters.as_ref().map(|it| it.inner.as_ref()));
            scope.maybe_many::<Func, _>(node.body.as_ref().map(|it| it.inner.as_ref()));
        })
    }
}

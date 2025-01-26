use crate::function::Func;
use crate::types::TyParam;
use kodept_ast::derive_node;
use kodept_ast::external::Component;
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::Name;
use kodept_ast::syntax_tree::prelude::{ASTBuilder, Pool};
use kodept_rlt::new_types::TypeName;
use kodept_rlt::prelude::{Enum, Struct};

#[derive(Debug, PartialEq, Component)]
pub enum EnumDecl {
    Stack,
    Heap,
}

#[derive(Debug, PartialEq, Component)]
pub struct StructDecl;

#[derive(Debug, PartialEq, Component)]
pub struct EnumConst;

derive_node!(EnumDecl {
    relations = [children EnumConst,],
    properties = [require Name,]
});

derive_node!(StructDecl {
    relations = [
        children TyParam,
        children Func,
    ],
    properties = [require Name,]
});

derive_node!(EnumConst {
    relations = [],
    properties = [require Name,]
});

impl FromSyntax for EnumDecl {
    type Syntax = Enum;

    fn from_syntax(
        node: &Self::Syntax,
        source: impl CodeHolder,
        builder: &Pool,
    ) -> ASTBuilder<Self> {
        let (kind, id, rest) = match node {
            Enum::Stack { id, contents, .. } => (EnumDecl::Stack, id, contents),
            Enum::Heap { id, contents, .. } => (EnumDecl::Heap, id, contents),
        };
        let name = source.get_chunk_located(id);
        ASTBuilder::new(builder, kind)
            .with_property(Name { name })
            .with_children(source, builder, |scope| {
                scope.maybe_many(rest.as_ref().map(|it| it.inner.as_ref()))
            })
    }
}

impl FromSyntax for StructDecl {
    type Syntax = Struct;

    fn from_syntax(node: &Struct, source: impl CodeHolder, builder: &Pool) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(builder, StructDecl)
            .with_property(Name { name })
            .with_children(source, builder, |scope| {
                scope
                    .maybe_many::<TyParam, _>(node.parameters.as_ref().map(|it| it.inner.as_ref()));
                scope.maybe_many::<Func, _>(node.body.as_ref().map(|it| it.inner.as_ref()));
            })
    }
}

impl FromSyntax for EnumConst {
    type Syntax = TypeName;

    fn from_syntax(node: &Self::Syntax, source: impl CodeHolder, pool: &Pool) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(node);
        ASTBuilder::new(pool, EnumConst).with_property(Name { name })
    }
}

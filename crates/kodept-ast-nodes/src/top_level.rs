use crate::function::Func;
use crate::types::TyParams;
use crate::utils::wrap_ty_params;
use kodept_ast::{derive_node, relation};
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
    properties = [require Name,]
});
relation!(EnumDecl => children EnumConst);

derive_node!(StructDecl {
    properties = [require Name,]
});
relation!(StructDecl => child TyParams);
relation!(StructDecl => children Func);

derive_node!(EnumConst {
    properties = [require Name,]
});

impl FromSyntax for EnumDecl {
    type Syntax = Enum;

    fn from_syntax<'w>(
        node: &'w Self::Syntax,
        source: impl CodeHolder,
        pool: Pool<'w>,
    ) -> ASTBuilder<Self> {
        let (kind, id, rest) = match node {
            Enum::Stack { id, contents, .. } => (EnumDecl::Stack, id, contents),
            Enum::Heap { id, contents, .. } => (EnumDecl::Heap, id, contents),
        };
        let name = source.get_chunk_located(id);
        ASTBuilder::new(pool, kind)
            .with_property(Name(name))
            .with_children(source, pool, |scope| {
                scope.maybe_many(rest.as_ref().map(|it| it.inner.as_ref()))
            })
    }
}

impl FromSyntax for StructDecl {
    type Syntax = Struct;

    fn from_syntax<'w>(node: &'w Self::Syntax, source: impl CodeHolder, pool: Pool<'w>) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(pool, StructDecl)
            .with_property(Name(name))
            .with_children(source, pool, |scope| {
                if let Some(params) = &node.parameters {
                    wrap_ty_params(node, &params.inner, scope);
                }
                scope.maybe_many::<Func, _>(node.body.as_ref().map(|it| it.inner.as_ref()));
            })
    }
}

impl FromSyntax for EnumConst {
    type Syntax = TypeName;

    fn from_syntax<'w>(node: &'w Self::Syntax, source: impl CodeHolder, pool: Pool<'w>) -> ASTBuilder<Self> {
        let name = source.get_chunk_located(node);
        ASTBuilder::new(pool, EnumConst).with_property(Name(name))
    }
}

use crate::function::FuncDecl;
use crate::types::TyParams;
use bevy_ecs::prelude::{Bundle, Component};
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::{Name, SourceSpan};
use kodept_ast::syntax_tree::prelude::ASTBuilder;
use kodept_ast::{derive_node, relation};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::new_types::TypeName;
use kodept_rlt::prelude::{Enum, Struct};
use std::convert::identity;

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
relation!(StructDecl => optional TyParams);
relation!(StructDecl => children FuncDecl);

derive_node!(EnumConst {
    properties = [require Name,]
});

impl FromSyntax<Enum> for EnumDecl {
    type Bundle = impl Bundle;

    fn from_syntax(node: &Enum, source: impl CodeHolder) -> Self::Bundle {
        let (kind, id, rest) = match node {
            Enum::Stack { id, contents, .. } => (EnumDecl::Stack, id, contents),
            Enum::Heap { id, contents, .. } => (EnumDecl::Heap, id, contents),
        };
        let name = source.get_chunk_located(id);
        ASTBuilder::new(kind)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .with_opt_children(rest.as_ref().map(|it| it.inner.as_ref()), source)
            .build()
    }
}

impl FromSyntax<Struct> for StructDecl {
    type Bundle = impl Bundle;

    fn from_syntax(node: &Struct, source: impl CodeHolder) -> Self::Bundle {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(StructDecl)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .with_opt_dyn_child(
                node.parameters.as_ref(),
                source,
                move |it, spawner, source| {
                    spawner.spawn_raw(
                        ASTBuilder::new(TyParams)
                            .with_property(SourceSpan(it.left.0 + it.right.0))
                            .with_children(it.inner.as_ref(), source),
                        node,
                        identity,
                    )
                },
            )
            .with_opt_children::<_, FuncDecl, _>(
                node.body.as_ref().map(|it| it.inner.as_ref()),
                source,
            )
            .build()
    }
}

impl FromSyntax<TypeName> for EnumConst {
    type Bundle = impl Bundle;

    fn from_syntax(node: &TypeName, source: impl CodeHolder) -> Self::Bundle {
        let name = source.get_chunk_located(node);
        ASTBuilder::new(EnumConst)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.0.into()))
            .build()
    }
}

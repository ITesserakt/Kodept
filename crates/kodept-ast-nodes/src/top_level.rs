use crate::function::Func;
use crate::types::TyParams;
use bevy_ecs::prelude::{Bundle, Component};
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::Name;
use kodept_ast::syntax_tree::experimental::ASTBuilder;
use kodept_ast::{derive_node, relation};
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
relation!(StructDecl => child TyParams);
relation!(StructDecl => children Func);

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
            .with_property(Name(name))
            .with_opt_children(rest.as_ref().map(|it| it.inner.as_ref()), source)
            .build()
    }
}

impl FromSyntax<Struct> for StructDecl {
    type Bundle = impl Bundle;

    fn from_syntax(node: &Struct, source: impl CodeHolder) -> Self::Bundle {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(StructDecl)
            .with_property(Name(name))
            .with_dyn_child(&node.parameters, source, move |it, spawner, source| {
                spawner.spawn_raw(
                    ASTBuilder::new(TyParams)
                        .with_opt_children(it.as_ref().map(|it| it.inner.as_ref()), source),
                    node,
                    identity,
                )
            })
            .with_opt_children::<_, Func, _>(node.body.as_ref().map(|it| it.inner.as_ref()), source)
            .build()
    }
}

impl FromSyntax<TypeName> for EnumConst {
    type Bundle = impl Bundle;

    fn from_syntax(node: &TypeName, source: impl CodeHolder) -> Self::Bundle {
        let name = source.get_chunk_located(node);
        ASTBuilder::new(EnumConst).with_property(Name(name)).build()
    }
}

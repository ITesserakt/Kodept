use crate::term::ReferenceContext;
use crate::utils::unwrap_type;
use bevy_ecs::prelude::{Bundle, Component};
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::{Name, SourceSpan};
use kodept_ast::syntax_tree::prelude::ASTBuilder;
use kodept_ast::{derive_node, relation, Str};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::new_types;
use kodept_rlt::prelude::{Tuple, TypedParameter, UntypedParameter};

#[derive(Debug, PartialEq, Component)]
pub struct Ty {
    pub context: ReferenceContext,
    pub ident: Str,
}

#[derive(Debug, PartialEq, Component)]
pub struct ProdTy;

#[derive(Debug, PartialEq, Component)]
pub struct TyParam;

#[derive(Debug, PartialEq, Component)]
pub struct NonTyParam;

derive_node!(Ty);

derive_node!(ProdTy);
relation!(ProdTy => children Ty);
relation!(ProdTy => children ProdTy);

derive_node!(TyParam {
    properties = [require Name,]
});
relation!(TyParam => optional Ty);
relation!(TyParam => optional ProdTy);

derive_node!(NonTyParam {
    properties = [require Name,]
});

impl FromSyntax<new_types::TypeName> for Ty {
    type Bundle = impl Bundle;
    type Error = crate::Error;

    fn from_syntax(
        node: &new_types::TypeName,
        source: impl CodeHolder,
    ) -> Result<Self::Bundle, Self::Error> {
        let name = source.get_chunk_located(node);
        Ok(ASTBuilder::new(Ty {
            context: ReferenceContext::empty(false),
            ident: name,
        })
        .with_property(SourceSpan(node.0.into()))
        .build())
    }
}

impl FromSyntax<UntypedParameter> for NonTyParam {
    type Bundle = impl Bundle;
    type Error = crate::Error;

    fn from_syntax(
        node: &UntypedParameter,
        source: impl CodeHolder,
    ) -> Result<Self::Bundle, Self::Error> {
        let name = source.get_chunk_located(&node.id);
        Ok(ASTBuilder::new(NonTyParam)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .build())
    }
}

impl FromSyntax<TypedParameter> for TyParam {
    type Bundle = impl Bundle;
    type Error = crate::Error;

    fn from_syntax(
        node: &TypedParameter,
        source: impl CodeHolder,
    ) -> Result<Self::Bundle, Self::Error> {
        let name = source.get_chunk_located(&node.id);
        Ok(ASTBuilder::new(TyParam)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_child(&node.parameter_type, source, unwrap_type)?
            .build())
    }
}

impl FromSyntax<Tuple> for ProdTy {
    type Bundle = impl Bundle;
    type Error = crate::Error;

    fn from_syntax(node: &Tuple, source: impl CodeHolder) -> Result<Self::Bundle, Self::Error> {
        Ok(ASTBuilder::new(ProdTy)
            .with_property(SourceSpan(node.0.left.0 + node.0.right.0))
            .with_dyn_children(node.0.inner.as_ref(), |it, spawner| {
                unwrap_type(it, spawner, source)
            })?
            .build())
    }
}

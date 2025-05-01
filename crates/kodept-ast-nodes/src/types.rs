use crate::utils::unwrap_type;
use bevy_ecs::prelude::{Bundle, Component};
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::Name;
use kodept_ast::syntax_tree::experimental::ASTBuilder;
use kodept_ast::{derive_node, relation};
use kodept_rlt::new_types;
use kodept_rlt::prelude::{Tuple, TypedParameter, UntypedParameter};

#[derive(Debug, PartialEq, Component)]
pub struct Ty;

#[derive(Debug, PartialEq, Component)]
pub struct ProdTy;

#[derive(Debug, PartialEq, Component)]
pub struct TyParam;

#[derive(Debug, PartialEq, Component)]
pub struct NonTyParam;

#[derive(Debug, PartialEq, Component)]
pub struct Params;
#[derive(Debug, PartialEq, Component)]
pub struct TyParams;

derive_node!(Params);
relation!(Params => children TyParam);
relation!(Params => children NonTyParam);

derive_node!(TyParams);
relation!(TyParams => children TyParam);

derive_node!(Ty {
    properties = [require Name,]
});

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

    fn from_syntax(node: &new_types::TypeName, source: impl CodeHolder) -> Self::Bundle {
        let name = source.get_chunk_located(node);
        ASTBuilder::new(Ty).with_property(Name(name)).build()
    }
}

impl FromSyntax<UntypedParameter> for NonTyParam {
    type Bundle = impl Bundle;

    fn from_syntax(node: &UntypedParameter, source: impl CodeHolder) -> Self::Bundle {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(NonTyParam)
            .with_property(Name(name))
            .build()
    }
}

impl FromSyntax<TypedParameter> for TyParam {
    type Bundle = impl Bundle;

    fn from_syntax(node: &TypedParameter, source: impl CodeHolder) -> Self::Bundle {
        let name = source.get_chunk_located(&node.id);
        ASTBuilder::new(TyParam)
            .with_property(Name(name))
            .with_dyn_child(&node.parameter_type, source, unwrap_type)
            .build()
    }
}

impl FromSyntax<Tuple> for ProdTy {
    type Bundle = impl Bundle;

    fn from_syntax(node: &Tuple, source: impl CodeHolder) -> Self::Bundle {
        ASTBuilder::new(ProdTy)
            .with_dyn_children(node.0.inner.as_ref(), |it, spawner| {
                unwrap_type(it, spawner, source)
            })
            .build()
    }
}

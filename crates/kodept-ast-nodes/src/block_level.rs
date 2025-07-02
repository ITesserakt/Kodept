use crate::code_flow::IfExpr;
use crate::expression::{App, BinExpr, Exprs, Lambda, UnExpr};
use crate::literal::{Literal, Tuple};
use crate::term::Ref;
use crate::types::{ProdTy, Ty};
use crate::utils::{unwrap_operation, unwrap_type};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::Component;
use kodept_ast::prelude::{CodeHolder, FromSyntax};
use kodept_ast::properties::{Name, SourceSpan};
use kodept_ast::syntax_tree::prelude::ASTBuilder;
use kodept_ast::{derive_node, relation};
use kodept_rlt::exported::SpanBounds;
use kodept_rlt::prelude::{InitializedVariable, Variable};

#[derive(Debug, PartialEq, Component)]
pub enum VarDecl {
    Immutable,
    Mutable,
}

#[derive(Debug, PartialEq, Component)]
pub struct InitVar;

derive_node!(VarDecl {
    properties = [require Name,]
});
relation!(VarDecl => optional Ty);
relation!(VarDecl => optional ProdTy);

derive_node!(InitVar);
relation!(InitVar => child VarDecl);
relation!(InitVar => optional Exprs);
relation!(InitVar => optional App);
relation!(InitVar => optional Lambda);
relation!(InitVar => optional IfExpr);
relation!(InitVar => optional BinExpr);
relation!(InitVar => optional UnExpr);
relation!(InitVar => optional Ref);
relation!(InitVar => optional Ty);
relation!(InitVar => optional Literal);
relation!(InitVar => optional Tuple);

impl FromSyntax<Variable> for VarDecl {
    type Bundle = impl Bundle;

    fn from_syntax(node: &Variable, source: impl CodeHolder) -> Self::Bundle {
        let (kind, id, ty) = match node {
            Variable::Immutable {
                id, assigned_type, ..
            } => (VarDecl::Immutable, id, assigned_type),
            Variable::Mutable {
                id, assigned_type, ..
            } => (VarDecl::Mutable, id, assigned_type),
        };
        let name = source.get_chunk_located(id);
        ASTBuilder::new(kind)
            .with_property(Name::new(name))
            .with_property(SourceSpan(node.bounds()))
            .with_dyn_children(ty.as_ref().map(|it| &it.1), |it, spawner| {
                unwrap_type(it, spawner, source)
            })
            .build()
    }
}

impl FromSyntax<InitializedVariable> for InitVar {
    type Bundle = impl Bundle;

    fn from_syntax(node: &InitializedVariable, source: impl CodeHolder) -> Self::Bundle {
        ASTBuilder::new(InitVar)
            .with_property(SourceSpan(node.bounds()))
            .with_child::<_, VarDecl, _>(&node.variable, source)
            .with_dyn_child(&node.expression, source, unwrap_operation)
            .build()
    }
}

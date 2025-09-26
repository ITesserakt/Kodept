use std::convert::Infallible;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};
use kodept_ast_nodes::types::Ty;
use kodept_inference::{
    prelude::{DefaultExecutor, Infer},
    r#type::MonomorphicType,
    traits::TypeInfer,
};

use crate::{
    typing::{TypeInferHandler, Typed},
};

impl<'a> TypeInfer<&'a Ty> for TypeInferHandler<'_, '_> {
    type Error = Infallible;
    type Output = MonomorphicType;

    fn apply<'b>(&mut self, expr: &'a Ty) -> Infer<'b, &'a Ty, Self>
    where
        &'a Ty: 'b,
    {
        let str = expr.ident.to_string();
        Infer::done(MonomorphicType::Constant(str.into()))
    }
}

pub(super) fn system(
    query: Query<(Entity, &Ty)>,
    mut handler: TypeInferHandler,
    mut commands: Commands,
) {
    let mut executor = DefaultExecutor::default();

    for (id, ty) in query.iter() {
        let infer = handler.infer_eagerly(ty, &mut executor).unwrap();
        commands.entity(id).insert(Typed(infer));
    }
}

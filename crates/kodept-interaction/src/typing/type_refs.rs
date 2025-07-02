use std::convert::Infallible;

use bevy_ecs::{
    entity::Entity,
    name::Name,
    system::{Commands, Query},
};
use kodept_ast::prelude::{NodeId, PropertyQuery};
use kodept_ast_nodes::types::Ty;
use kodept_inference::{
    prelude::{DefaultExecutor, Infer},
    r#type::MonomorphicType,
    traits::TypeInfer,
};

use crate::{
    done,
    typing::{TypeInferHandler, Typed},
};

type TyInferParams<'a> = (NodeId<Ty>, &'a Ty);

impl<'a> TypeInfer<TyInferParams<'a>>
    for TypeInferHandler<'_, '_, PropertyQuery<'_, '_, Ty, Name>>
{
    type Error = Infallible;
    type Output = MonomorphicType;

    fn apply<'b>(&mut self, expr: TyInferParams<'a>) -> Infer<'b, TyInferParams<'a>, Self>
    where
        TyInferParams<'a>: 'b,
    {
        let name = self.additional_queries.get_required(expr.0).unwrap();
        let str = name.as_str().to_string();
        Infer::done(MonomorphicType::Constant(str.into()))
    }
}

pub(super) fn system(
    query: Query<(Entity, &Ty)>,
    mut handler: TypeInferHandler<PropertyQuery<Ty, Name>>,
    mut commands: Commands,
) -> crate::Result<Infallible> {
    let mut executor = DefaultExecutor::default();

    for (id, ty) in query.iter() {
        let infer = handler
            .infer_eagerly((id.into(), ty), &mut executor)
            .unwrap();
        commands.entity(id).insert(Typed(infer));
    }
    done()
}

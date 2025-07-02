use std::convert::Infallible;

use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{ResMut, StaticSystemParam, SystemParam},
};
use kodept_inference::{
    assumption::AssumptionSet, constraint::Constraint, r#type::MonomorphicType,
};

use crate::{wrapper::InteractionExt, Interaction};

mod literals;
mod type_refs;

pub struct TypeInferPass;

#[derive(Debug, Resource, Default)]
struct TVarGen(usize);

#[derive(SystemParam)]
struct TypeInferHandler<'w, 's, Q = ()>
where
    Q: SystemParam + 'static,
{
    tvar_id_gen: ResMut<'w, TVarGen>,
    additional_queries: StaticSystemParam<'w, 's, Q>,
}

#[derive(Debug, Component)]
#[component(immutable)]
struct Typed(MonomorphicType);

struct PartialInfer {
    assumptions: AssumptionSet,
    constraints: Vec<Constraint>,
    immediate_type: MonomorphicType,
}

impl PartialInfer {
    fn from_ty(ty: impl Into<MonomorphicType>) -> Self {
        Self {
            assumptions: AssumptionSet::default(),
            constraints: Vec::new(),
            immediate_type: ty.into(),
        }
    }
}

impl Interaction for TypeInferPass {
    type Error = Infallible;

    fn install(ctx: &mut crate::Ctx) {
        ctx.immediate_exclusive(|w| w.init_resource::<TVarGen>());
        ctx.register(Self::wrap_system(literals::system));
        ctx.register(Self::wrap_system(type_refs::system));
    }
}

impl std::ops::Deref for Typed {
    type Target = MonomorphicType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

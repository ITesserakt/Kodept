use crate::utils::{Ctx, Disposable, Interaction, wrap_system};
use bevy_ecs::component::Component;
use kodept_inference::r#type::MonomorphicType;

mod function;
mod literals;
mod refs;
mod type_refs;

pub struct TypeInferPass;

#[derive(Debug, Component)]
#[component(immutable)]
pub(crate) struct Typed(pub MonomorphicType);

impl Interaction for TypeInferPass {
    fn install(ctx: &mut Ctx) -> impl Disposable + use<> {
        ctx.register(wrap_system(Self::name(), literals::system));
        ctx.register(wrap_system(Self::name(), type_refs::ty_system));
        ctx.register(wrap_system(Self::name(), type_refs::prod_ty_system));
        ctx.register(wrap_system(Self::name(), refs::system));
    }
}

impl std::ops::Deref for Typed {
    type Target = MonomorphicType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

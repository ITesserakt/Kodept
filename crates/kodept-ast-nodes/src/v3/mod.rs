use bevy_ecs::prelude::Component;
use kodept_ast::properties::Node;

mod convert;
mod dispatch;
mod links;
mod tags;
mod types;

pub use tags::*;
pub use types::*;

#[derive(Debug, PartialEq, Component)]
#[require(Node::of::<Self>())]
pub struct Modules;

#[cfg(feature = "reflection")]
pub(super) fn register_reflection_info(
    registry: &mut kodept_ast::resource::reflection::DebugRegistry,
) {
    registry.register::<UserType>();
    registry.register::<TypeCtor<Resolved>>();
    registry.register::<TypeCtor<Unresolved>>();
    registry.register::<PrimType>();
    registry.register::<Module>();
    registry.register::<UserFunction<Option<Unresolved>>>();
    registry.register::<UserFunction<Resolved>>();
    registry.register::<ForeignFunction<Unresolved>>();
    registry.register::<ForeignFunction<Resolved>>();
    registry.register::<AnonFunction<Option<Unresolved>>>();
    registry.register::<AnonFunction<Resolved>>();
    registry.register::<Variable<Option<Unresolved>>>();
    registry.register::<Variable<Resolved>>();
    registry.register::<Block>();
    registry.register::<Block<true>>();
    registry.register::<Value<Unresolved>>();
    registry.register::<Value<Resolved>>();
    registry.register::<Literal>();
    registry.register::<Tuple>();
    registry.register::<Call>();
    registry.register::<If>();
    registry.register::<Branch>();
    registry.register::<Otherwise>();
    registry.register::<Link>();
}

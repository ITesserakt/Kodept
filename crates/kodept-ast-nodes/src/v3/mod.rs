mod convert;
mod dispatch;
mod links;
mod tags;
mod types;

pub use tags::*;
pub use types::*;

#[cfg(feature = "reflection")]
pub(super) fn register_reflection_info(
    registry: &mut kodept_ast::resource::reflection::DebugRegistry,
) {
    registry.register::<UserType>();
    registry.register::<PrimType>();
    registry.register::<Module>();
    registry.register::<Block>();
    registry.register::<Block<true>>();
    registry.register::<Value>();
    registry.register::<Literal>();
    registry.register::<Tuple>();
    registry.register::<Call>();
    registry.register::<If>();
    registry.register::<Branch>();
    registry.register::<Otherwise>();
    registry.register::<Link>();
    registry.register::<UserFunction>();
    registry.register::<Variable>();
    registry.register::<ForeignFunction>();
    registry.register::<AnonFunction>();
    registry.register::<ValueCtor>();
    registry.register::<Param>();

    registry.register::<TypeAnnotation>();
    registry.register::<ResolvedTypeAnnotation>();
    registry.register::<UnresolvedType>();
    registry.register::<ResolvedType>();
}

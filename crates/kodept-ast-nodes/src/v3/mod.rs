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
    registry.register::<UserFunction<TypeAnnotation>>();
    registry.register::<UserFunction<ResolvedTypeAnnotation>>();
    registry.register::<Variable<TypeAnnotation>>();
    registry.register::<Variable<ResolvedTypeAnnotation>>();
    registry.register::<ForeignFunction<UnresolvedType>>();
    registry.register::<ForeignFunction<ResolvedType>>();
    registry.register::<AnonFunction<TypeAnnotation>>();
    registry.register::<AnonFunction<ResolvedTypeAnnotation>>();
    registry.register::<ValueCtor<UnresolvedType>>();
    registry.register::<ValueCtor<ResolvedType>>();
}

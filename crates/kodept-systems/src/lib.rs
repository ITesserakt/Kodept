pub mod configs;
pub mod global;
pub mod loader;
pub mod per_file;
pub mod source;
pub mod utils;

#[cfg(feature = "reflection")]
pub fn register_reflection_info(registry: &mut kodept_ast::resource::reflection::DebugRegistry) {
    registry.register::<per_file::prelude::InModule>();
    registry.register::<per_file::prelude::SymbolTable>();
}

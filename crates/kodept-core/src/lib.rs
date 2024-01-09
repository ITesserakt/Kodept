use std::any::type_name;

pub mod code_point;
pub mod code_source;
pub mod file_relative;
pub mod loader;
pub mod structure;

pub trait Named {
    fn name(&self) -> &'static str {
        type_name::<Self>()
    }
}

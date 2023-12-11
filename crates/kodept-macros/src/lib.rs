use tracing::warn;

pub mod analyzer;
pub mod analyzers;
pub mod erased;
pub mod error;
pub mod traits;
pub mod transformer;
// pub mod transformers;

pub fn warn_about_broken_rlt<T>() {
    warn!(
        expected = std::any::type_name::<T>(),
        "Skipping some checks because node in RLT either doesn't exist or has different type."
    );
}

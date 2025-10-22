// pub mod lint_;
// mod normalize;
// mod report;
// mod scope;
// mod symbol;
// mod typing;
// mod utils;
pub mod configs;
pub mod global;
pub mod loader;
pub mod per_file;
pub mod source;
pub mod utils;
pub mod lint;
// pub mod prelude {
//     pub use super::report::install_reporting_support;
//     pub use super::scope::builder::ScopeBuildingPass;
//     pub use super::scope::references::ReferenceResolverPass;
//     pub use super::symbol::interaction::{DuplicatedSymbolError, ExtractSymbolsPass};
//     pub use super::typing::TypeInferPass;
//
//     pub use super::utils::{
//         install_system_completion_introspection_support, Disposable, Interaction,
//         SystemCompletionEvent,
//     };
// }

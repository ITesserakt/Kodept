#![feature(try_trait_v2)]
#![feature(min_specialization)]

pub use constcat::concat_slices;
pub use paste::paste;
pub use ref_cast::RefCast;

#[allow(unused_imports)]
pub(crate) use graph::with_children;
#[allow(unused_imports)]
pub(crate) use macros::implementation::node;
pub use node_properties::Uninit;

pub use self::node::{
    block_level::*, code_flow::*, expression::*, file::*, function::*, literal::*, term::*,
    top_level::*, types::*,
};

pub mod ast_builder;
pub mod graph;
mod macros;
mod node;
mod node_properties;
pub mod rlt_accessor;
pub mod traits;
pub mod utils;
pub mod visit_side;

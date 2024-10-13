pub use constcat::concat_slices;
pub use paste::paste;
pub use ref_cast::RefCast;

#[allow(unused_imports)]
pub(crate) use graph::with_children;
#[allow(unused_imports)]
pub(crate) use macros::implementation::{node, parent_definition};

pub use self::node::{
    block_level::*, code_flow::*, expression::*, file::*, function::*, literal::*, term::*,
    top_level::*, types::*,
};
pub use uninit::Uninit;

pub mod graph;
mod macros;
mod node;
pub mod rlt_accessor;
pub mod traits;
mod uninit;
pub mod interning;

pub mod visit_side {
    use derive_more::IsVariant;

    #[derive(IsVariant, Clone, Ord, PartialOrd, Eq, PartialEq, Copy, Debug)]
    #[repr(u8)]
    pub enum VisitSide {
        Entering,
        Exiting,
        Leaf,
    }
}

pub mod utils {
    use derive_more::From;

    #[derive(Default, Debug, From)]
    pub enum Skip<E> {
        Failed(E),
        #[default]
        #[from(ignore)]
        Skipped,
    }
}

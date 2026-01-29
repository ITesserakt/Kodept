mod buffer;
mod builder_v4;
pub mod children;
mod iteration;
mod modification;

pub mod prelude {
    pub use super::iteration::{AllNodesQuery, AllNodesQueryIter, NodeSlot};
}

#[deprecated]
pub mod experimental {
    pub use super::buffer::{Buffer, RefBuffer};
    pub use super::builder_v4::{
        AnonSpawner, Constructed, Constructing, ConstructingNode, NodeBuilder, NodeSpawner,
        RelatedNodeSpawner, Spawned, SpawnedNode, Spawner, SpawnerNode,
    };
    pub use super::modification::{ChainedNodeModification, NodeModification};
}

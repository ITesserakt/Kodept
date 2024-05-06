#![feature(impl_trait_in_assoc_type)]

use petgraph::{Directed, Undirected};

pub mod export;
mod graph;
mod key;
mod parts;
mod subgraph;

pub type DiGraph<N, E = ()> = graph::Graph<N, E, Directed>;
pub type UnGraph<N, E = ()> = graph::Graph<N, E, Undirected>;

pub type SubDiGraph<N, E> = subgraph::SubGraph<N, E, Directed>;
pub type SubUnGraph<N, E> = subgraph::SubGraph<N, E, Undirected>;

pub use key::Key;
pub use parts::EdgeKey;
pub use parts::NodeKey;

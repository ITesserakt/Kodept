use crate::make_ast_node_adaptor;
use crate::*;
use derive_more::{From, TryInto};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

type Identity<T> = T;

make_ast_node_adaptor!(GenericASTNode, lifetimes: [], Identity, configs: [
    derive(Debug, PartialEq, From, TryInto),
    try_into(owned, ref, ref_mut),
    cfg_attr(feature = "serde", derive(Serialize, Deserialize))
]);

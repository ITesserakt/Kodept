use crate::graph::traits::Identifiable;
use crate::graph::SyntaxTree;
use crate::make_ast_node_adaptor;
use crate::*;
use derive_more::{From, TryInto};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "size-of")]
use size_of::SizeOf;

type Identity<T> = T;

make_ast_node_adaptor!(GenericASTNode, lifetimes: [], Identity, configs: [
    derive(Debug, PartialEq, From, TryInto),
    try_into(owned, ref, ref_mut),
    cfg_attr(feature = "serde", derive(Serialize, Deserialize)),
    cfg_attr(feature = "size-of", derive(SizeOf))
]);

pub trait NodeWithParent {
    type Parent;
}

pub trait Node: Identifiable {
    fn parent<'b>(&self, tree: &'b SyntaxTree) -> &'b Self::Parent
    where
        Self: NodeWithParent + 'static,
        for<'a> &'a Self::Parent: TryFrom<&'a GenericASTNode>,
    {
        let id = self.get_id();
        tree.parent_of(id)
    }

    fn parent_mut<'b>(&self, tree: &'b mut SyntaxTree) -> &'b mut Self::Parent
    where
        Self: NodeWithParent + 'static,
        for<'a> &'a mut Self::Parent: TryFrom<&'a mut GenericASTNode>,
    {
        let id = self.get_id();
        tree.parent_of_mut(id)
    }
}

pub unsafe trait NodeUnion
where
    Self: From<GenericASTNode>,
    for<'a> &'a Self: TryFrom<&'a GenericASTNode>,
    for<'a> &'a mut Self: TryFrom<&'a mut GenericASTNode>,
{
    fn contains(node: &GenericASTNode) -> bool;
}

unsafe impl NodeUnion for GenericASTNode {
    #[inline]
    fn contains(_node: &GenericASTNode) -> bool {
        true
    }
}

pub(crate) mod macros {
    #[macro_export]
    macro_rules! wrapper {
        ($(#[$config:meta])* $vis:vis wrapper $wrapper:ident($inner:ty);) => {
            $(#[$config])*
            #[repr(transparent)]
            pub struct $wrapper($inner);

            impl<'a> TryFrom<&'a GenericASTNode> for &'a $wrapper {
                type Error = <&'a $inner as TryFrom<&'a GenericASTNode>>::Error;

                fn try_from(value: &'a GenericASTNode) -> Result<Self, Self::Error> {
                    let node: &$inner = value.try_into()?;
                    Ok(unsafe { std::mem::transmute(node) })
                }
            }

            impl<'a> TryFrom<&'a mut GenericASTNode> for &'a mut $wrapper {
                type Error = <&'a mut $inner as TryFrom<&'a mut GenericASTNode>>::Error;

                fn try_from(value: &'a mut GenericASTNode) -> Result<Self, Self::Error> {
                    let node: &mut $inner = value.try_into()?;
                    Ok(unsafe { std::mem::transmute(node) })
                }
            }
        };
        ($(#[$config:meta])* $vis:vis wrapper $wrapper:ident {
            $($name:ident($t:ty) = $variants:path$(,)*)*
        }) => {
            wrapper!($(#[$config])* $vis wrapper $wrapper(GenericASTNode););
            unsafe impl $crate::graph::generic_node::NodeUnion for $wrapper {
                fn contains(node: &GenericASTNode) -> bool {
                    match node {
                        $($variants(_) => true,)*
                        _ => false
                    }
                }
            }

            impl $wrapper {
                paste::paste! {
                    $(
                    pub fn [<as_ $name>](&self) -> Option<&$t> {
                        match self {
                            $wrapper($variants(x)) => Some(x),
                            _ => None,
                        }
                    }
                    pub fn [<as_ $name _mut>](&mut self) -> Option<&mut $t> {
                        match self {
                            $wrapper($variants(x)) => Some(x),
                            _ => None
                        }
                    }
                    )*
                }
            }
        }
    }
}

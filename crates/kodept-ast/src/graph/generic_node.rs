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

/// # Safety
/// Implement only for `#repr(transparent)` structs
pub unsafe trait NodeUnion: Sized {
    fn contains(node: &GenericASTNode) -> bool;

    #[inline]
    fn wrap(node: &GenericASTNode) -> &Self {
        debug_assert!(Self::contains(node));
        unsafe { std::mem::transmute(node) }
    }

    #[inline]
    fn wrap_mut(node: &mut GenericASTNode) -> &mut Self {
        debug_assert!(Self::contains(node));
        unsafe { std::mem::transmute(node) }
    }
}

unsafe impl NodeUnion for GenericASTNode {
    #[inline]
    fn contains(_node: &GenericASTNode) -> bool {
        true
    }

    #[inline]
    fn wrap(node: &GenericASTNode) -> &Self {
        node
    }

    #[inline]
    fn wrap_mut(node: &mut GenericASTNode) -> &mut Self {
        node
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

                #[inline]
                fn try_from(value: &'a GenericASTNode) -> Result<Self, Self::Error> {
                    let node: &$inner = value.try_into()?;
                    Ok(unsafe { std::mem::transmute(node) })
                }
            }

            impl<'a> TryFrom<&'a mut GenericASTNode> for &'a mut $wrapper {
                type Error = <&'a mut $inner as TryFrom<&'a mut GenericASTNode>>::Error;

                #[inline]
                fn try_from(value: &'a mut GenericASTNode) -> Result<Self, Self::Error> {
                    let node: &mut $inner = value.try_into()?;
                    Ok(unsafe { std::mem::transmute(node) })
                }
            }

            #[cfg(feature = "size-of")]
            impl size_of::SizeOf for $wrapper where $inner: size_of::SizeOf {
                fn size_of_children(&self, context: &mut size_of::Context) {
                    self.0.size_of_children(context)
                }
            }
        };
        ($(#[$config:meta])* $vis:vis wrapper $wrapper:ident {
            $($name:ident($t:ty) = $variants:pat $(if $variant_if:expr)? => $variant_expr:expr$(,)*)*
        }) => {
            wrapper!($(#[$config])* $vis wrapper $wrapper(GenericASTNode););
            unsafe impl $crate::graph::generic_node::NodeUnion for $wrapper {
                fn contains(node: &GenericASTNode) -> bool {
                    #[allow(unused_variables)]
                    #[allow(unreachable_patterns)]
                    match node {
                        $($variants $(if $variant_if)? => true,)*
                        _ => false
                    }
                }
            }

            impl $wrapper {
                paste::paste! {
                    $(
                    #[inline]
                    pub fn [<as_ $name>](&self) -> Option<&$t> {
                        match self {
                            $wrapper($variants) $(if $variant_if)? => $variant_expr,
                            _ => None,
                        }
                    }
                    #[inline]
                    pub fn [<as_ $name _mut>](&mut self) -> Option<&mut $t> {
                        match self {
                            $wrapper($variants) $(if $variant_if)? => $variant_expr,
                            _ => None
                        }
                    }
                    )*
                }
            }
        }
    }
}

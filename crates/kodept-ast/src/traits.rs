use kodept_core::structure::span::CodeHolder;

use crate::graph::{AnyNode, NodeId, SubSyntaxTree};

pub trait Identifiable: Sized {
    fn get_id(&self) -> NodeId<Self>;
}

impl<T: crate::graph::Identifiable> Identifiable for T {
    fn get_id(&self) -> NodeId<Self> {
        <Self as crate::graph::Identifiable>::get_id(self)
    }
}

#[allow(clippy::wrong_self_convention)]
pub trait AsEnum {
    type Enum;

    fn as_enum(self) -> Self::Enum;
}

pub trait PopulateTree<'a> {
    type Root: Into<AnyNode>;

    fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root>;
}

#[cfg(feature = "parallel")]
pub mod parallel {
    use crate::graph::tags::ChildTag;
    use crate::graph::{AnyNode, HasChildrenMarker, Identifiable, SubSyntaxTree};
    use crate::traits::PopulateTree;
    use crate::{FileDecl, Uninit};
    use kodept_core::structure::rlt;
    use kodept_core::structure::span::CodeHolder;
    use rayon::prelude::*;
    use std::sync::mpsc::channel;

    pub struct Parallelize<'a, T, const TAG: ChildTag>(pub &'a T);

    pub trait Bound<'a> {
        type Node: Into<AnyNode>;

        fn convert(self) -> Uninit<'a, Self::Node>;
    }

    pub trait SingleChildrenFamily {
        type Child;

        fn children(&self) -> &[Self::Child];
    }

    impl<'a, T, U, A, B, const TAG: ChildTag> PopulateTree<'a> for Parallelize<'a, T, TAG>
    where
        T: SingleChildrenFamily<Child = U>,
        U: Sync + 'a,
        &'a T: Bound<'a, Node = A>,
        &'a U: PopulateTree<'a, Root = B>,
        A: Identifiable + Into<AnyNode>,
        A: HasChildrenMarker<B, TAG>,
        B: Send,
    {
        type Root = A;

        fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
            let (sx, rx) = channel();
            std::thread::scope(move |s| {
                let children = self.0.children();
                s.spawn(move || {
                    children
                        .into_par_iter()
                        .map(|it| it.convert(context))
                        .for_each_with(sx, |sx, it| sx.send(it).unwrap());
                });
                let mut root = SubSyntaxTree::new(self.0.convert());
                for child in rx {
                    root.attach_subtree(child);
                }
                root
            })
        }
    }

    impl SingleChildrenFamily for rlt::File {
        type Child = rlt::Module;

        fn children(&self) -> &[Self::Child] {
            self.0.as_ref()
        }
    }

    impl<'a> Bound<'a> for &'a rlt::File {
        type Node = FileDecl;

        fn convert(self) -> Uninit<'a, Self::Node> {
            FileDecl::uninit().with_rlt(self)
        }
    }
}

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

    pub struct Parallelize<T, const TAG: ChildTag>(pub T);

    pub trait Bound {
        type Node: Into<AnyNode>;

        fn convert(self) -> Uninit<Self::Node>;
    }

    pub trait SingleChildrenFamily {
        type Child;

        fn children(&self) -> &[Self::Child];
    }

    impl<'a, T, U, A, B, const TAG: ChildTag> PopulateTree for &'a Parallelize<T, TAG>
    where
        T: SingleChildrenFamily<Child = U>,
        &T: Bound<Node = A>,
        &U: PopulateTree<Root = B> + Sync + 'static,
        A: Identifiable + Into<AnyNode>,
        A: HasChildrenMarker<B, TAG>,
        B: Send,
    {
        type Root = A;

        fn convert(self, context: &impl CodeHolder) -> SubSyntaxTree<'a, Self::Root> {
            let mut root = SubSyntaxTree::new(self.0.convert());
            let subtrees = self
                .0
                .children()
                .par_iter()
                .map(|it| it.convert(context))
                .collect_vec_list();
            for vec in subtrees {
                for subtree in vec {
                    root.attach_subtree(subtree)
                }
            }
            root
        }
    }

    impl SingleChildrenFamily for rlt::File {
        type Child = rlt::Module;

        fn children(&self) -> &[Self::Child] {
            self.0.as_ref()
        }
    }
    
    impl Bound for &rlt::File {
        type Node = FileDecl;

        fn convert(self) -> Uninit<Self::Node> {
            FileDecl::uninit().with_rlt(self)
        }
    }
}

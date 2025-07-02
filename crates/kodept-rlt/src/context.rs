use crate::new_types::{Symbol, TypeName};
use std::collections::VecDeque;

pub struct StartsFromRoot;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Context {
    Global {
        colon: Symbol,
    },
    Local,
    Inner {
        parent: Box<Context>,
        needle: TypeName,
    },
}

impl Context {
    pub fn is_global(&self) -> bool {
        let mut current = self;

        loop {
            match current {
                Context::Global { .. } => return true,
                Context::Local => return false,
                Context::Inner { parent, .. } => {
                    current = parent.as_ref();
                    continue;
                }
            }
        }
    }

    pub fn unfold(&self) -> (Option<StartsFromRoot>, Vec<&TypeName>) {
        let mut refs = VecDeque::new();
        let mut current = self;
        loop {
            match current {
                Context::Global { .. } => return (Some(StartsFromRoot), Vec::from(refs)),
                Context::Local => return (None, Vec::from(refs)),
                Context::Inner { needle, parent } => {
                    refs.push_front(needle);
                    current = &*parent;
                }
            }
        }
    }
}

#[cfg(feature = "arbitrary")]
mod arb {
    use crate::new_types::{Symbol, TypeName};
    use crate::prelude::Context;
    use kodept_core::code_point::CodePoint;
    use proptest::prelude::{any, Arbitrary, BoxedStrategy, Just, Strategy};
    use proptest::prop_oneof;

    impl Arbitrary for Context {
        type Parameters = ();

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            fn points_with_offset() -> impl Strategy<Value = CodePoint> {
                any::<CodePoint>()
                    .prop_filter("At least 2 letters required for global contexts", |it| {
                        it.offset > 2
                    })
            }

            let leaf = prop_oneof![
                points_with_offset().prop_map(|it| Context::Global {
                    colon: Symbol::from(it)
                }),
                Just(Context::Local)
            ];
            leaf.prop_recursive(4, 20, 5, |inner| {
                (points_with_offset(), inner).prop_map(|it| Context::Inner {
                    needle: TypeName::from(it.0),
                    parent: Box::new(it.1),
                })
            })
            .boxed()
        }

        type Strategy = BoxedStrategy<Self>;
    }
}

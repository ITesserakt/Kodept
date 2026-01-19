use crate::new_types::{Keyword, Symbol, TypeName};
use crate::prelude::TopLevelNode;
use derive_more::Constructor;
use kodept_core::code_point::{CodePoint, Span};
use kodept_core::structure::{Located, SpanBounds};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub enum Module {
    Global {
        keyword: Keyword,
        id: TypeName,
        flow: Symbol,
        rest: Box<[TopLevelNode]>,
    },
    Ordinary {
        keyword: Keyword,
        id: TypeName,
        lbrace: Symbol,
        rest: Box<[TopLevelNode]>,
        rbrace: Symbol,
    },
}

#[derive(Debug, Clone, PartialEq, Constructor)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct File(
    #[cfg_attr(feature = "arbitrary", proptest(strategy = "arb::gen_modules()"))] pub Box<[Module]>,
);

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary))]
pub struct RLT(pub File);

impl Module {
    pub fn get_keyword(&self) -> &Keyword {
        match self {
            Module::Global { keyword, .. } => keyword,
            Module::Ordinary { keyword, .. } => keyword,
        }
    }
}

impl Located for Module {
    #[inline]
    fn location(&self) -> CodePoint {
        self.get_keyword().location()
    }
}

impl Located for File {
    #[inline]
    fn location(&self) -> CodePoint {
        CodePoint::new(0, 0)
    }
}

impl SpanBounds for File {
    #[inline]
    fn bounds(&self) -> Span {
        CodePoint::single_point(0)
            + self.0.first().map(|it| it.bounds())
            + self.0.last().map(|it| it.bounds())
    }
}

impl SpanBounds for Module {
    #[inline]
    fn bounds(&self) -> Span {
        match self {
            Module::Global {
                keyword,
                flow,
                rest,
                ..
            } => keyword.0 + flow.0 + rest.last().map(|it| it.bounds()),
            Module::Ordinary {
                keyword, rbrace, ..
            } => keyword.0 + rbrace.0,
        }
    }
}

#[cfg(feature = "arbitrary")]
mod arb {
    use crate::file::Module;
    use crate::new_types::{Keyword, Symbol, TypeName};
    use crate::prelude::TopLevelNode;
    use proptest::collection::vec;
    use proptest::prelude::{Strategy, any};
    use proptest::prop_oneof;

    pub(super) fn gen_modules() -> impl Strategy<Value = Box<[Module]>> {
        prop_oneof![
            (
                any::<Keyword>(),
                any::<TypeName>(),
                any::<Symbol>(),
                vec(any::<TopLevelNode>(), 0..20)
            )
                .prop_map(|it| {
                    Box::from([Module::Global {
                        keyword: it.0,
                        id: it.1,
                        flow: it.2,
                        rest: it.3.into_boxed_slice(),
                    }])
                }),
            vec(
                (
                    any::<Keyword>(),
                    any::<TypeName>(),
                    any::<Symbol>(),
                    vec(any::<TopLevelNode>(), 0..20),
                    any::<Symbol>()
                ),
                0..5
            )
            .prop_map(|it| {
                it.into_iter()
                    .map(|it| Module::Ordinary {
                        keyword: it.0,
                        id: it.1,
                        lbrace: it.2,
                        rest: it.3.into_boxed_slice(),
                        rbrace: it.4,
                    })
                    .collect()
            })
        ]
    }
}

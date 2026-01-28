use bevy_ecs::prelude::Resource;
use kodept_core::Freeze;
use kodept_core::structure::SpanBounds;
use kodept_rlt::exported::Span;
use kodept_rlt::prelude as rlt;
use kodept_rlt::prelude::RLT;
use kodept_rlt::traversal::{ErasedNodeBorrow, ErasedNodePtr, SyntaxNode};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomPinned;
use std::pin::Pin;

#[derive(PartialEq, Copy, Clone)]
pub struct LexemeId(ErasedNodePtr);

#[derive(Debug)]
struct PinnedRLT {
    inner: Freeze<RLT>,
    _phantom: PhantomPinned,
}

#[derive(Debug, Resource)]
pub struct SyntaxResolver {
    tree: Pin<Box<PinnedRLT>>,
}

pub enum LookupError {
    NotFound,
    WrongType,
}

impl LexemeId {
    #[inline]
    pub(crate) const fn from(value: ErasedNodePtr) -> Self {
        Self(value)
    }

    #[inline]
    pub fn from_syntax(value: &impl SyntaxNode) -> Self {
        Self::from(ErasedNodePtr::new(value))
    }
}

impl SyntaxResolver {
    pub fn build(tree: RLT) -> Self {
        let this = Self {
            tree: Box::pin(PinnedRLT {
                inner: Freeze::new(tree),
                _phantom: Default::default(),
            }),
        };

        Self { ..this }
    }

    pub fn root(&self) -> (&rlt::File, LexemeId) {
        (
            &self.tree.inner.0,
            LexemeId(ErasedNodePtr::new(&self.tree.inner.0)),
        )
    }

    #[allow(unsafe_code)]
    fn borrow_ptr(&self, node: &ErasedNodePtr) -> ErasedNodeBorrow<'_> {
        // SAFETY: lifetime of elements is tied to `self` and RLT is pinned
        unsafe { node.borrow() }
    }

    pub fn get_span(&self, id: LexemeId) -> Span {
        self.borrow_ptr(&id.0).bounds()
    }

    pub fn try_get_unknown(&self, id: LexemeId) -> Option<ErasedNodeBorrow<'_>> {
        Some(self.borrow_ptr(&id.0))
    }

    pub fn try_get<T: SyntaxNode>(&self, id: LexemeId) -> Result<&T, LookupError> {
        match self.try_get_unknown(id) {
            Some(x) => x.try_cast().ok_or(LookupError::WrongType),
            None => Err(LookupError::NotFound),
        }
    }
}

impl Debug for LexemeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LexemeId")
            .field("ptr", &self.0)
            .finish_non_exhaustive()
    }
}

use crate::Str;
use crate::message::Diagnostic;
use kodept_core::code_point::CodePoint;
use kodept_core::either::Either;
use std::any::type_name_of_val;
use std::convert::Infallible;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;

pub trait SpannedReportMessage: Into<Diagnostic> {
    #[deprecated]
    fn with_node_location(self, location: CodePoint) -> impl IntoSpannedReportMessage;
}

/// Determines whether a message will break execution
#[derive(Debug)]
pub enum MessageBehaviour {
    FailFast {
        /// Should return an explanation why does associated message cannot be reported for multiple nodes
        reason: Str,
    },
    /// Continue execution
    Suppress,
}

pub trait IntoSpannedReportMessage {
    type Message: SpannedReportMessage;

    #[inline]
    fn behaviour(&self) -> MessageBehaviour {
        MessageBehaviour::Suppress
    }

    #[inline]
    fn code(&self) -> u32 {
        let type_name = type_name_of_val(self);
        let mut hasher = DefaultHasher::new();
        type_name.hash(&mut hasher);
        let hash = hasher.finish();
        hash as u32 ^ (hash >> 32) as u32
    }

    fn into_message(self) -> Self::Message;
}

#[inline(always)]
pub fn ad_hoc_message<T>(f: impl FnOnce() -> T) -> impl IntoSpannedReportMessage<Message = T>
where
    T: SpannedReportMessage,
{
    struct Helper<F, T>(F, PhantomData<T>);

    impl<F, T> IntoSpannedReportMessage for Helper<F, T>
    where
        T: SpannedReportMessage,
        F: FnOnce() -> T,
    {
        type Message = T;

        fn into_message(self) -> Self::Message {
            self.0()
        }
    }

    Helper(f, PhantomData)
}

impl MessageBehaviour {
    pub fn fail_fast(reason: impl Into<Str>) -> Self {
        Self::FailFast {
            reason: reason.into(),
        }
    }

    /// Makes pipeline fail without any reason
    pub fn fail_really_fast() -> Self {
        Self::FailFast { reason: "".into() }
    }
}

impl IntoSpannedReportMessage for Infallible {
    type Message = Diagnostic;

    #[inline(always)]
    fn behaviour(&self) -> MessageBehaviour {
        match *self {}
    }

    #[inline(always)]
    fn code(&self) -> u32 {
        match *self {}
    }

    #[inline(always)]
    fn into_message(self) -> Self::Message {
        match self {}
    }
}

impl<A, B, M: SpannedReportMessage> IntoSpannedReportMessage for Either<A, B>
where
    A: IntoSpannedReportMessage<Message = M>,
    B: IntoSpannedReportMessage<Message = M>,
{
    type Message = M;

    #[inline(always)]
    fn behaviour(&self) -> MessageBehaviour {
        match self {
            Either::Left(x) => IntoSpannedReportMessage::behaviour(x),
            Either::Right(x) => IntoSpannedReportMessage::behaviour(x),
        }
    }

    #[inline(always)]
    fn code(&self) -> u32 {
        match self {
            Either::Left(x) => IntoSpannedReportMessage::code(x),
            Either::Right(x) => IntoSpannedReportMessage::code(x),
        }
    }

    #[inline(always)]
    fn into_message(self) -> Self::Message {
        match self {
            Either::Left(x) => IntoSpannedReportMessage::into_message(x),
            Either::Right(x) => IntoSpannedReportMessage::into_message(x),
        }
    }
}

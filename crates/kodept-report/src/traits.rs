use crate::Str;
use crate::message::{Diagnostic, ReportMessage, Severity};
use kodept_core::code_point::CodePoint;
use kodept_core::either::Either;
use std::any::type_name_of_val;
use std::convert::Infallible;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;

pub trait Message: Into<Diagnostic> {
    #[deprecated]
    fn with_node_location(self, location: CodePoint) -> impl IntoMessage;
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

pub trait IntoMessage {
    type Message: Message;

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
pub fn lazy_message<T>(f: impl FnOnce() -> T) -> impl IntoMessage<Message = T>
where
    T: Message,
{
    struct Helper<F, T>(F, PhantomData<T>);

    impl<F, T> IntoMessage for Helper<F, T>
    where
        T: Message,
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

impl IntoMessage for Infallible {
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

impl<A, B, M: Message> IntoMessage for Either<A, B>
where
    A: IntoMessage<Message = M>,
    B: IntoMessage<Message = M>,
{
    type Message = M;

    #[inline(always)]
    fn behaviour(&self) -> MessageBehaviour {
        match self {
            Either::Left(x) => IntoMessage::behaviour(x),
            Either::Right(x) => IntoMessage::behaviour(x),
        }
    }

    #[inline(always)]
    fn code(&self) -> u32 {
        match self {
            Either::Left(x) => IntoMessage::code(x),
            Either::Right(x) => IntoMessage::code(x),
        }
    }

    #[inline(always)]
    fn into_message(self) -> Self::Message {
        match self {
            Either::Left(x) => IntoMessage::into_message(x),
            Either::Right(x) => IntoMessage::into_message(x),
        }
    }
}

impl IntoMessage for std::io::Error {
    type Message = ReportMessage;

    fn into_message(self) -> Self::Message {
        ReportMessage::new(Severity::Error, self.to_string())
    }
}

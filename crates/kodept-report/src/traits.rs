use std::any::type_name_of_val;
use std::error::Error;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;
use kodept_core::code_point::CodePoint;
use crate::message::{Diagnostic, ReportMessage, Severity};
use crate::Str;

pub trait SpannedReportMessage: Into<Diagnostic> {
    fn with_node_location(self, location: CodePoint) -> impl IntoSpannedReportMessage;
}

/// Determines whether a message will break execution
#[derive(Debug)]
pub enum MessageBehaviour {
    FailFast {
        /// Should return an explanation why does associated message cannot be reported for multiple nodes
        reason: Str,
    },
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
}

impl<E: Error> IntoSpannedReportMessage for E {
    type Message = ReportMessage;

    fn into_message(self) -> Self::Message {
        ReportMessage::new(Severity::Error, self.to_string())
    }
}

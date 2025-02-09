use crate::message::{ReportMessage, Severity};
use crate::traits::{IntoSpannedReportMessage, MessageBehaviour};
use std::any::Any;

#[derive(Debug)]
pub struct CompilerCrash {
    inner: Box<dyn Any + Send>,
}

impl IntoSpannedReportMessage for CompilerCrash {
    type Message = ReportMessage;

    fn behaviour(&self) -> MessageBehaviour {
        MessageBehaviour::fail_fast("Unrecoverable error happened")
    }

    fn code(&self) -> u32 {
        u32::MAX
    }

    fn into_message(self) -> Self::Message {
        let message = match self.inner.downcast::<String>() {
            Ok(x) => *x,
            Err(x) => match x.downcast::<&str>() {
                Ok(x) => x.to_string(),
                Err(_) => "Unknown error".to_string(),
            },
        };
        ReportMessage::new(Severity::Error, message)
    }
}

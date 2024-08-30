use derive_more::Display;
use std::any::Any;
use std::error::Error;
use std::fmt::Formatter;
use std::sync::Mutex;

pub mod compiler_crash;
pub mod report;
pub mod report_collector;
pub mod traits;

#[derive(Debug, Default)]
pub struct ErrorReported {
    message: Option<Box<Mutex<dyn Any + Send>>>,
    cause: Option<anyhow::Error>,
}

impl ErrorReported {
    pub fn with_cause<E: Error + Send + Sync + 'static>(error: E) -> Self {
        Self {
            message: None,
            cause: Some(anyhow::Error::from(error)),
        }
    }

    pub fn with_message<M: Any + Send>(self, message: M) -> Self {
        Self {
            message: Some(Box::new(Mutex::new(message))),
            ..self
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}

impl Error for ErrorReported {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.cause {
            None => None,
            Some(x) => Some(x.as_ref()),
        }
    }
}

impl Display for ErrorReported {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = self.message.as_ref() {
            let guard = msg.lock().expect("Error message is poisoned");
            if let Some(msg) = guard.downcast_ref::<String>() {
                write!(f, "Compilation failed due to produced errors: {msg}")
            } else if let Some(msg) = guard.downcast_ref::<&str>() {
                write!(f, "Compilation failed due to produced errors: {msg}")
            } else if let Some(msg) = guard.downcast_ref::<&dyn Display>() {
                write!(f, "Compilation failed due to produced errors: {msg}")
            } else {
                write!(f, "Compilation failed due to produced errors")
            }
        } else {
            write!(f, "Compilation failed due to produced errors")
        }
    }
}

use std::error::Error;
use derive_more::{Display};

pub mod compiler_crash;
pub mod report;
pub mod report_collector;
pub mod traits;

#[derive(Debug, Display, Default)]
#[display("Compilation failed due to produced errors")]
pub struct ErrorReported {
    cause: Option<Box<dyn Error + Send + Sync + 'static>>
}

impl ErrorReported {
    pub fn with_cause<E: Error + Send + Sync + 'static>(error: E) -> Self {
        Self {
            cause: Some(Box::new(error)),
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
use derive_more::Display;
use std::error::Error;
use std::fmt::Formatter;

pub mod compiler_crash;
pub mod report;
pub mod report_collector;
pub mod traits;

#[derive(Debug, Default)]
pub struct ErrorReported {
    cause: Option<anyhow::Error>,
}

impl ErrorReported {
    pub fn with_cause<E: Error + Send + Sync + 'static>(error: E) -> Self {
        Self {
            cause: Some(anyhow::Error::from(error)),
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
        write!(f, "Compilation failed due to produced errors")
    }
}

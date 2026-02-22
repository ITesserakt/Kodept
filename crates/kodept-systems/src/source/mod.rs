use crate::loader::{Loader, LoadingError};
use crate::source::collection::Sources;
use crate::source::unloaded::CodeSourceError;
use derive_more::{Display, Error, From};
use kodept_report::message::Severity;
use kodept_report::prelude::{IntoSpannedReportMessage, ReportMessage};

pub mod collection;
pub mod loaded;
pub mod unloaded;

#[derive(Debug, Error, From, Display)]
pub enum SourcesLoadingError {
    Opening(LoadingError),
    Loading(CodeSourceError),
}

pub fn load_each_source(loader: Loader) -> Result<Sources, SourcesLoadingError> {
    let unloaded_sources = loader.into_sources()?;
    let mut sources = Sources::new();
    for source in unloaded_sources {
        sources.insert(source)?;
    }
    Ok(sources)
}

impl IntoSpannedReportMessage for SourcesLoadingError {
    type Message = ReportMessage;

    fn into_message(self) -> Self::Message {
        ReportMessage::new(Severity::Error, self.to_string())
    }
}

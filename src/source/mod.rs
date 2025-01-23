use thiserror::Error;
use crate::loader::{Loader, LoadingError};
use crate::source::collection::Sources;
use crate::source::loaded::CodeSourceError;

pub mod unloaded;
pub mod loaded;
pub mod collection;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SourcesLoadingError {
    Opening(#[from] LoadingError),
    Loading(#[from] CodeSourceError)
}

pub fn load_each_source(loader: Loader) -> Result<Sources, SourcesLoadingError> {
    let unloaded_sources = loader.into_sources()?;
    let mut sources = Sources::new();
    for source in unloaded_sources {
        sources.insert(source)?;
    }
    Ok(sources)
} 

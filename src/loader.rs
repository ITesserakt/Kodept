use std::borrow::Cow;
use std::env::current_dir;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use itertools::Itertools;
use kodept_core::code_source::CodeSource;
use thiserror::Error;
use tracing::warn;

pub enum Loader {
    File(Vec<(File, PathBuf)>),
    Memory(Vec<String>),
}

#[derive(Error, Debug)]
pub enum LoadingError {
    #[error("Provided path should be absolute")]
    StartingPathNotAbsolute,
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("No input files")]
    NoInput,
}

pub struct LoaderBuilder<'p> {
    starting_path: Cow<'p, Path>,
    extension: Cow<'p, OsStr>,
    accept_any_extension: bool,
    #[allow(dead_code)]
    cache_extension: &'p OsStr,
}

impl Default for LoaderBuilder<'static> {
    fn default() -> Self {
        Self {
            starting_path: Cow::Owned(
                current_dir().expect("Cannot get absolute path starting from here"),
            ),
            extension: Cow::Borrowed(OsStr::new("kd")),
            accept_any_extension: false,
            cache_extension: OsStr::new("kdc"),
        }
    }
}

impl<'p> LoaderBuilder<'p> {
    #[must_use]
    pub fn with_starting_path<P: Into<Cow<'p, Path>>>(self, path: P) -> Self {
        Self {
            starting_path: path.into(),
            ..self
        }
    }

    #[must_use]
    pub fn with_extension<S: Into<Cow<'p, OsStr>>>(self, extension: S) -> Self {
        Self {
            extension: extension.into(),
            ..self
        }
    }

    #[must_use]
    pub fn with_any_source_extension(self) -> Self {
        Self {
            accept_any_extension: true,
            ..self
        }
    }

    pub fn build(self) -> Result<Loader, LoadingError> {
        let sources = if self.starting_path.is_dir() {
            if !self.starting_path.is_absolute() {
                return Err(LoadingError::StartingPathNotAbsolute);
            }

            self.starting_path
                .read_dir()
                .map_err(LoadingError::IOError)?
                .filter_ok(|it| {
                    if !it.path().is_file() {
                        false
                    } else if self.accept_any_extension {
                        true
                    } else {
                        it.path()
                            .extension()
                            .is_some_and(|ext| ext == self.extension)
                    }
                })
                .filter_map_ok(|it| {
                    let file = match File::open(it.path()) {
                        Ok(f) => f,
                        Err(e) => {
                            warn!("Skipping file {0} because: {1}", it.path().display(), e);
                            return None;
                        }
                    };

                    Some((file, it.path()))
                })
                .try_collect()?
        } else if self.starting_path.is_file()
            && self
                .starting_path
                .extension()
                .is_some_and(|ext| ext == self.extension)
        {
            vec![(
                File::open(&self.starting_path)?,
                self.starting_path.into_owned(),
            )]
        } else {
            vec![]
        };

        if sources.is_empty() {
            Err(LoadingError::NoInput)
        } else {
            Ok(Loader::File(sources))
        }
    }
}

impl Loader {
    #[must_use]
    pub fn file<'b>() -> LoaderBuilder<'b> {
        LoaderBuilder::default()
    }

    pub fn from_single_snippet<S: Into<String>>(text: S) -> Self {
        Self::Memory(vec![text.into()])
    }

    pub fn from_text(text: impl IntoIterator<Item = String>) -> Self {
        Self::Memory(text.into_iter().collect())
    }

    fn generate_scratch_name(index: usize) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        format!("scratch-#{index}-{time}.kd", time = timestamp.as_millis())
    }

    #[must_use]
    pub fn into_sources(self) -> Vec<CodeSource> {
        match self {
            Loader::File(sources) => sources
                .into_iter()
                .map(|it| CodeSource::file(it.1, it.0))
                .collect(),
            Loader::Memory(sources) => sources
                .into_iter()
                .enumerate()
                .map(|(i, it)| CodeSource::memory(Self::generate_scratch_name(i), it))
                .collect(),
        }
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::Memory(Default::default())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::env::temp_dir;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::Path;

    use crate::loader::Loader;

    #[test]
    fn test_load_text_from_scratch() {
        let text = "Hello world";
        let loader = Loader::from_single_snippet(text);
        let mut sources = loader.into_sources();

        assert_eq!(sources.len(), 1);
        let mut source = sources.pop().unwrap();
        let mut output = String::new();
        source.read_to_string(&mut output).unwrap();
        assert_eq!(output, text);
    }

    fn suite<P: AsRef<Path> + ?Sized>(file: &mut File, filepath: &P) {
        let loader = Loader::file().with_starting_path(filepath.as_ref()).build();
        assert!(loader.is_ok());
        let loader = loader.unwrap();

        let text = "Hello world";
        write!(file, "{0}", text).unwrap();

        let mut sources = loader.into_sources();
        assert_eq!(sources.len(), 1);
        let mut source = sources.pop().unwrap();
        let mut output = String::new();
        source.read_to_string(&mut output).unwrap();
        assert_eq!(output, text);
    }

    #[test]
    #[ignore]
    fn test_load_from_file_by_folder() {
        let mut file = tempfile::Builder::new().suffix(".kd").tempfile().unwrap();

        suite(file.as_file_mut(), &temp_dir());
        file.close().unwrap();
    }

    #[test]
    fn test_load_from_file_by_concrete_file() {
        let mut file = tempfile::Builder::new().suffix(".kd").tempfile().unwrap();
        let path = file.path().to_owned();

        suite(file.as_file_mut(), &path);
        file.close().unwrap();
    }

    #[test]
    fn test_load_any_temp_file() {
        let _ = tempfile::tempfile();
        let loader = Loader::file()
            .with_starting_path(&temp_dir())
            .with_any_source_extension()
            .build()
            .unwrap();

        let sources = loader.into_sources();
        assert!(!sources.is_empty())
    }
}

use std::borrow::Cow;
use std::env::current_dir;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use derive_more::{Display, Error, From};
use crate::source::unloaded::{CodeSource, CodeSourceError};
use tracing::debug;

#[derive(Debug)]
pub enum Loader {
    File(Vec<(File, PathBuf)>),
    Memory(Vec<String>),
}

#[derive(Error, Debug, From, Display)]
pub enum LoadingError {
    #[display("Provided path should be absolute")]
    StartingPathNotAbsolute,
    #[display("Provided path does not exists")]
    InputDoesNotExists,
    #[display("IO error: {_0}")]
    IOError(std::io::Error),
    #[display("Cannot map file: {_0}")]
    MapError(CodeSourceError),
    #[display("No input files")]
    NoInput,
}

pub struct LoaderBuilder<'p> {
    starting_path: Cow<'p, Path>,
    extension: Cow<'p, OsStr>,
    accept_any_extension: bool,
    #[allow(dead_code)]
    cache_extension: &'p OsStr,
}

const MAP_FILESIZE: u64 = 20 * 1024 * 1024; // 20 MB

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
        match self.starting_path.try_exists() {
            Ok(true) => {}
            Ok(false) => return Err(LoadingError::InputDoesNotExists),
            Err(io) => return Err(LoadingError::IOError(io)),
        };
        let sources = if self.starting_path.is_dir() {
            self.starting_path
                .read_dir()
                .map_err(LoadingError::IOError)?
                .filter_map(|it| it.ok())
                .filter(|it| {
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
                .map(|it| match File::open(it.path()) {
                    Ok(f) => Ok((f, it.path())),
                    Err(e) => Err(e),
                })
                .collect::<Result<_, _>>()?
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

    fn mmap_if_needed(mut file: File, path: PathBuf) -> Result<CodeSource, LoadingError> {
        let size = file.seek(SeekFrom::End(0))?;
        file.rewind()?;
        if size > MAP_FILESIZE {
            debug!("Using mmap to load file {}", path.display());
            Ok(CodeSource::mmap(path, file, Some(size))?)
        } else {
            Ok(CodeSource::file(path, file))
        }
    }
    
    pub fn into_sources(self) -> Result<Vec<CodeSource>, LoadingError> {
        match self {
            Loader::File(sources) => sources
                .into_iter()
                .map(|it| Self::mmap_if_needed(it.0, it.1))
                .collect(),
            Loader::Memory(sources) => Ok(sources.into_iter().map(CodeSource::memory).collect()),
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
        let mut sources = loader.into_sources().unwrap();

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

        let mut sources = loader.into_sources().unwrap();
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
    #[ignore]
    fn test_load_any_temp_file() {
        let _ = tempfile::tempfile();
        let loader = Loader::file()
            .with_starting_path(temp_dir())
            .with_any_source_extension()
            .build()
            .unwrap();

        let sources = loader.into_sources().unwrap();
        assert!(!sources.is_empty())
    }
}

use crate::common_iter::CommonIter;
use crate::read_code_source::ReadCodeSource;
use codespan_reporting::files::{Error, Files};
use kodept_core::code_source::CodeSource;
use kodept_core::file_relative::CodePath;
use std::collections::hash_map::IntoValues;
use std::collections::HashMap;
use std::ops::Range;
use tracing::error;

pub type FileId = u16;

pub struct SourceFiles {
    contents: HashMap<FileId, ReadCodeSource>,
}

impl SourceFiles {
    pub fn from_sources(sources: Vec<CodeSource>) -> Self {
        let map = sources
            .into_iter()
            .filter_map(|it| {
                let path = it.path();
                match it.try_into() {
                    Ok(source) => Some(source),
                    Err(e) => {
                        error!(?path, "Cannot read source, I/O error: {e}.");
                        None
                    }
                }
            })
            .enumerate()
            .map(|(idx, it)| (idx as FileId, it))
            .collect();
        Self { contents: map }
    }

    pub fn into_common_iter(self) -> impl CommonIter<Item = ReadCodeSource> {
        #[cfg(not(feature = "parallel"))] {
            self.into_iter()
        }
        #[cfg(feature = "parallel")] {
            use rayon::prelude::IntoParallelIterator;
            self.into_par_iter()
        }
    }
}

impl IntoIterator for SourceFiles {
    type Item = ReadCodeSource;
    type IntoIter = IntoValues<FileId, ReadCodeSource>;

    fn into_iter(self) -> Self::IntoIter {
        self.contents.into_values()
    }
}

impl<'a> Files<'a> for SourceFiles {
    type FileId = FileId;
    type Name = CodePath;
    type Source = &'a str;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, Error> {
        match self.contents.get(&id) {
            None => Err(Error::FileMissing),
            Some(x) => Ok(x.path()),
        }
    }

    fn source(&'a self, id: Self::FileId) -> Result<Self::Source, Error> {
        match self.contents.get(&id) {
            None => Err(Error::FileMissing),
            Some(x) => Ok(x.contents()),
        }
    }

    fn line_index(&'a self, id: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        match self.contents.get(&id) {
            None => Err(Error::FileMissing),
            Some(x) => x.line_index((), byte_index),
        }
    }

    fn line_range(&'a self, id: Self::FileId, line_index: usize) -> Result<Range<usize>, Error> {
        match self.contents.get(&id) {
            None => Err(Error::FileMissing),
            Some(x) => x.line_range((), line_index),
        }
    }
}

#[cfg(feature = "parallel")]
mod parallel {
    use crate::read_code_source::ReadCodeSource;
    use crate::source_files::{FileId, SourceFiles};
    use rayon::collections::hash_map::IntoIter as HashMapIntoIter;
    use rayon::iter::Map;
    use rayon::prelude::*;

    impl IntoParallelIterator for SourceFiles {
        type Iter = Map<
            HashMapIntoIter<FileId, ReadCodeSource>,
            fn((FileId, ReadCodeSource)) -> ReadCodeSource,
        >;
        type Item = ReadCodeSource;

        fn into_par_iter(self) -> Self::Iter {
            self.contents.into_par_iter().map(|(_, it)| it)
        }
    }
}

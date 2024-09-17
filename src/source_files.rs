use crate::common_iter::CommonIter;
use crate::read_code_source::ReadCodeSource;
use codespan_reporting::files::{Error, Files};
use kodept_core::code_source::CodeSource;
use kodept_core::file_name::FileName;
use kodept_core::Freeze;
use kodept_macros::context::{FileDescriptor, FileId};
use std::collections::HashMap;
use std::ops::{Deref, Range};
use std::sync::Arc;
use tracing::error;
use yoke::Yoke;

#[derive(Debug, Clone)]
pub struct SourceView {
    pub id: Freeze<FileId>,
    source: Yoke<&'static ReadCodeSource, Arc<SourceFiles>>,
}

#[derive(Debug)]
pub struct SourceFiles {
    contents: HashMap<FileId, ReadCodeSource>,
}

impl Deref for SourceView {
    type Target = ReadCodeSource;

    fn deref(&self) -> &Self::Target {
        self.source.get()
    }
}

impl SourceView {
    pub fn all_files(&self) -> &SourceFiles {
        self.source.backing_cart()
    }
    
    pub fn describe(&self) -> FileDescriptor {
        FileDescriptor {
            name: self.source.get().path(),
            id: *self.id,
        }
    }
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

    pub fn into_common_iter<'a>(self: &'a Arc<Self>) -> impl CommonIter<Item =SourceView> + 'a {
        #[cfg(not(feature = "parallel"))]
        {
            self.contents.keys().copied().map(|id| SourceView {
                id: Freeze::new(id),
                source: Yoke::attach_to_cart(self.clone(), |this| &this.contents[&id]),
            })
        }
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;

            self.contents
                .par_iter()
                .map(|it| *it.0)
                .map(|id| SourceView {
                    id: Freeze::new(id),
                    source: Yoke::attach_to_cart(self.clone(), |this| &this.contents[&id]),
                })
        }
    }
}

impl<'a> Files<'a> for SourceFiles {
    type FileId = FileId;
    type Name = FileName;
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

impl<'a> Files<'a> for SourceView {
    type FileId = FileId;
    type Name = FileName;
    type Source = &'a str;

    fn name(&'a self, _: Self::FileId) -> Result<Self::Name, Error> {
        self.source.get().name(())
    }

    fn source(&'a self, _: Self::FileId) -> Result<Self::Source, Error> {
        self.source.get().source(())
    }

    fn line_index(&'a self, _: Self::FileId, byte_index: usize) -> Result<usize, Error> {
        self.source.get().line_index((), byte_index)
    }

    fn line_range(&'a self, _: Self::FileId, line_index: usize) -> Result<Range<usize>, Error> {
        self.source.get().line_range((), line_index)
    }
}

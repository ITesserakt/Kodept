use crate::prelude::{Source, TryReadCode};
use crate::read_code_source::ReadSource;
use codespan_reporting::files::{Error, Files};
use kodept_core::file_name::FileName;
use kodept_core::Freeze;
use kodept_report::{FileDescriptor, FileId};
use std::collections::HashMap;
use std::ops::{Deref, Range};
use std::sync::Arc;
use yoke::Yoke;

#[derive(Debug)]
pub struct SourceView<Impl: 'static> {
    pub id: Freeze<FileId>,
    source: Yoke<&'static ReadSource<Impl>, Arc<SourceFiles<Impl>>>,
}

#[derive(Debug, Default)]
pub struct SourceFiles<Impl> {
    id_gen: FileId,
    contents: HashMap<FileId, ReadSource<Impl>>,
}

impl<Impl> Deref for SourceView<Impl> {
    type Target = ReadSource<Impl>;

    fn deref(&self) -> &Self::Target {
        self.source.get()
    }
}

impl<Impl> Clone for SourceView<Impl> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            source: self.source.clone(),
        }
    }
}

impl<Impl> SourceView<Impl> {
    pub fn all_files(&self) -> &SourceFiles<Impl> {
        self.source.backing_cart()
    }

    pub fn describe(&self) -> FileDescriptor {
        FileDescriptor {
            name: self.source.get().path().clone(),
            id: *self.id,
        }
    }
}

impl<Impl: 'static> SourceFiles<Impl> {
    pub fn new() -> Self {
        Self {
            id_gen: 0,
            contents: Default::default(),
        }
    }

    pub fn insert<T>(&mut self, source: T) -> Result<(), Impl::Error>
    where
        Impl: TryReadCode<T>,
    {
        let id = self.id_gen;
        self.contents.insert(id, Impl::try_read(source)?);
        self.id_gen = self.id_gen.checked_add(1).expect("Too many source files");
        Ok(())
    }

    pub fn collect(self: &Arc<Self>) -> Vec<SourceView<Impl>> {
        self.contents
            .keys()
            .copied()
            .map(move |id| SourceView {
                id: Freeze::new(id),
                source: Yoke::attach_to_cart(self.clone(), |this| &this.contents[&id]),
            })
            .collect()
    }
}

impl<'a, Impl> Files<'a> for SourceFiles<Impl>
where
    Impl: Source,
    Impl::Ref<'a>: AsRef<str>,
{
    type FileId = FileId;
    type Name = FileName;
    type Source = Impl::Ref<'a>;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, Error> {
        match self.contents.get(&id) {
            None => Err(Error::FileMissing),
            Some(x) => Ok(x.path().clone()),
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

impl<'a, Impl> Files<'a> for SourceView<Impl>
where
    Impl: Source,
    Impl::Ref<'a>: AsRef<str>,
{
    type FileId = ();
    type Name = FileName;
    type Source = Impl::Ref<'a>;

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

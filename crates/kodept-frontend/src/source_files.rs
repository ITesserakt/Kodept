use crate::prelude::Source;
use crate::read_code_source::{ReadSource, TryReadCode};
use bevy_ecs::prelude::Resource;
use codespan_reporting::files::{Error, Files};
use kodept_core::file_name::FileName;
use kodept_core::Freeze;
use kodept_report::{FileDescriptor, FileId};
use std::collections::HashMap;
use std::ops::{Deref, Range};
use std::sync::Arc;
use yoke::Yoke;

pub struct GlobalReports;

#[derive(Debug)]
pub struct SourceView<Impl: 'static> {
    pub id: Freeze<FileId>,
    source: Yoke<&'static ReadSource<Impl>, Arc<SourceFiles<Impl>>>,
}

#[derive(Debug, Resource)]
pub struct SourceFiles<Impl> {
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

impl Files<'static> for GlobalReports {
    type FileId = ();
    type Name = &'static str;
    type Source = &'static str;

    fn name(&'static self, _: Self::FileId) -> Result<Self::Name, Error> {
        Ok("<global level>")
    }

    fn source(&'static self, _: Self::FileId) -> Result<Self::Source, Error> {
        Err(Error::FileMissing)
    }

    fn line_index(&'static self, _: Self::FileId, _: usize) -> Result<usize, Error> {
        Err(Error::FileMissing)
    }

    fn line_range(&'static self, _: Self::FileId, _: usize) -> Result<Range<usize>, Error> {
        Err(Error::FileMissing)
    }
}

impl<Impl> SourceView<Impl> {
    pub fn all_files(&self) -> &SourceFiles<Impl> {
        self.source.backing_cart()
    }

    pub fn describe(&self) -> FileDescriptor {
        FileDescriptor {
            name: self.source.get().path(),
            id: *self.id,
        }
    }
}

impl<Impl: 'static> SourceFiles<Impl> {
    pub fn try_from_sources<T>(sources: impl IntoIterator<Item = T>) -> Result<Self, Impl::Error>
    where
        Impl: TryReadCode<T>,
    {
        let contents: Result<HashMap<_, _>, _> = sources
            .into_iter()
            .map(Impl::try_read)
            .enumerate()
            .map(|(idx, it)| match it {
                Ok(source) => {
                    let id = FileId::try_from(idx).expect("Too many source files");
                    Ok((id, source))
                }
                Err(e) => Err(e),
            })
            .collect();

        Ok(Self {
            contents: contents?,
        })
    }

    pub fn into_iter(self: &Arc<Self>) -> impl Iterator<Item = SourceView<Impl>> {
        self.contents.keys().copied().map(move |id| SourceView {
            id: Freeze::new(id),
            source: Yoke::attach_to_cart(self.clone(), |this| &this.contents[&id]),
        })
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

impl<'a, Impl> Files<'a> for SourceView<Impl>
where
    Impl: Source,
    Impl::Ref<'a>: AsRef<str>,
{
    type FileId = FileId;
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

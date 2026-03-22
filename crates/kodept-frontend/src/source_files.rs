use crate::read_code_source::{ReadSource, TryReadSource};
use kodept_core::Freeze;
use kodept_core::file_name::FileName;
use kodept_ecs::component::Component;
use kodept_ecs::exported::bevy_ecs;
use kodept_ecs::resource::Resource;
use kodept_report::files::external::{Error, Files};
use kodept_report::{FileDescriptor, FileId};
use std::collections::HashMap;
use std::ops::{Deref, Range};
use std::sync::Arc;
use yoke::Yoke;

#[derive(Debug, Component, Resource)]
pub struct SourceView {
    pub id: Freeze<FileId>,
    source: Yoke<&'static ReadSource, Arc<SourceFiles>>,
}

#[derive(Debug, Resource)]
pub struct CollectedSources {
    pub inner: Arc<SourceFiles>,
}

#[derive(Debug, Default)]
pub struct SourceFiles {
    contents: HashMap<FileId, ReadSource>,
}

impl Deref for SourceView {
    type Target = ReadSource;

    fn deref(&self) -> &Self::Target {
        self.source.get()
    }
}

impl Clone for SourceView {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            source: self.source.clone(),
        }
    }
}

impl SourceView {
    pub fn all_files(&self) -> &SourceFiles {
        self.source.backing_cart()
    }

    pub fn describe(&self) -> FileDescriptor {
        FileDescriptor::new(self.source.get().path().clone(), *self.id)
    }
}

impl SourceFiles {
    pub fn new() -> Self {
        Self {
            contents: HashMap::new(),
        }
    }

    pub fn insert<T>(&mut self, source: T) -> Result<(), T::Error>
    where
        T: TryReadSource,
    {
        let id = FileId::generate();
        self.contents.insert(id, source.try_read()?);
        Ok(())
    }

    pub fn collect(self: &Arc<Self>) -> Vec<SourceView> {
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

impl<'a> Files<'a> for SourceFiles {
    type FileId = FileId;
    type Name = FileName;
    type Source = &'a str;

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

impl<'a> Files<'a> for SourceView {
    type FileId = ();
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

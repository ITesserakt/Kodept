use kodept_core::code_point::CodePoint;
use kodept_core::file_name::FileName;
use kodept_core::structure::span::CodeHolder;
use kodept_report::files::external::{Error, Files};
use std::ops::Range;

#[derive(Debug)]
pub struct ReadSource<Impl = String> {
    source_contents: Impl,
    source_path: FileName,
    line_starts: Vec<usize>,
}

pub trait TryReadCode<From>: Sized {
    type Error;

    fn try_read(value: From) -> Result<ReadSource<Self>, Self::Error>;
}

pub trait Source
where
    Self: 'static,
{
    type Ref<'a>;

    fn as_ref(&self) -> Self::Ref<'_>;
    fn len(&self) -> usize;
    fn range(&self, value: Range<usize>) -> Self::Ref<'_>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait SyncSource
where
    Self: Send + Sync + 'static,
    Self: for<'a> Source<Ref<'a>: AsRef<str>>,
{
}

impl<T> SyncSource for T
where
    T: Send + Sync + 'static,
    T: for<'a> Source<Ref<'a>: AsRef<str>>,
{
}

impl Source for String {
    type Ref<'a> = &'a str;

    fn as_ref(&self) -> Self::Ref<'_> {
        self.as_str()
    }
    fn len(&self) -> usize {
        self.len()
    }

    fn range(&self, value: Range<usize>) -> Self::Ref<'_> {
        &self[value]
    }
}

impl<Impl> ReadSource<Impl> {
    pub fn new(inner: Impl, source_path: FileName, line_starts: Vec<usize>) -> Self {
        Self {
            source_contents: inner,
            source_path,
            line_starts,
        }
    }

    pub fn path(&self) -> &FileName {
        &self.source_path
    }

    pub fn contents(&self) -> Impl::Ref<'_>
    where
        Impl: Source,
    {
        self.source_contents.as_ref()
    }

    pub(crate) fn line_starts(&self) -> &[usize] {
        &self.line_starts
    }
}

impl<'a, Impl> CodeHolder for &'a ReadSource<Impl>
where
    Impl: Send + Sync + Source,
{
    type Str = Impl::Ref<'a>;

    fn get_chunk(self, at: CodePoint) -> Self::Str {
        self.source_contents.range(at.as_range())
    }
}

impl<Impl> ReadSource<Impl>
where
    Impl: Source,
{
    fn line_start(&self, line_index: usize) -> Result<usize, Error> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.line_starts().len()) {
            Ordering::Less => Ok(self
                .line_starts()
                .get(line_index)
                .cloned()
                .expect("failed despite previous check")),
            Ordering::Equal => Ok(self.source_contents.len()),
            Ordering::Greater => Err(Error::LineTooLarge {
                given: line_index,
                max: self.line_starts().len() - 1,
            }),
        }
    }
}

impl<'a, Impl> Files<'a> for ReadSource<Impl>
where
    Impl: Source,
    Impl::Ref<'a>: AsRef<str>,
{
    type FileId = ();
    type Name = FileName;
    type Source = Impl::Ref<'a>;

    fn name(&'a self, (): ()) -> Result<Self::Name, Error> {
        Ok(self.path().clone())
    }

    fn source(&'a self, (): ()) -> Result<Self::Source, Error> {
        Ok(self.contents())
    }

    fn line_index(&'a self, (): (), byte_index: usize) -> Result<usize, Error> {
        Ok(self
            .line_starts()
            .binary_search(&byte_index)
            .unwrap_or_else(|next_line| next_line - 1))
    }

    fn line_range(&'a self, (): (), line_index: usize) -> Result<Range<usize>, Error> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + 1)?;

        Ok(line_start..next_line_start)
    }
}

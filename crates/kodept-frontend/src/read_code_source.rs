use kodept_core::code_point::CodePoint;
use kodept_core::file_name::FileName;
use kodept_core::structure::span::CodeHolder;
use kodept_report::files::external::{Error, Files};
use memmap2::{Advice, Mmap};
use std::ops::Range;
use tracing::warn;
use yoke::Yoke;

#[derive(Debug)]
enum SourceBacking {
    /// Explicit storage in memory
    Explicit(String),
    /// Source is loaded implicitly via mmap
    ImplicitMMap(Yoke<&'static str, Box<Mmap>>),
}

#[derive(Debug)]
pub struct ReadSource {
    source_contents: SourceBacking,
    source_path: FileName,
    line_starts: Vec<usize>,
}

impl SourceBacking {
    fn implicit(mmap: Mmap) -> Result<Self, std::str::Utf8Error> {
        Ok(Self::ImplicitMMap(Yoke::try_attach_to_cart(
            Box::new(mmap),
            |it| std::str::from_utf8(it),
        )?))
    }

    #[inline]
    fn as_ref(&self) -> &str {
        match self {
            SourceBacking::Explicit(x) => x.as_str(),
            SourceBacking::ImplicitMMap(x) => x.get(),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            SourceBacking::Explicit(x) => x.len(),
            SourceBacking::ImplicitMMap(x) => x.get().len(),
        }
    }
}

pub trait TryReadSource {
    type Error;

    fn try_read(self) -> Result<ReadSource, Self::Error>;
}

impl ReadSource {
    fn get_line_starts(source: &str) -> impl Iterator<Item = usize> {
        std::iter::once(0)
            .chain(source.match_indices('\n').map(|it| it.0 + 1))
            .filter(|it| source.is_char_boundary(*it))
    }

    pub fn explicit(contents: String, source_path: FileName) -> Self {
        let line_starts = Self::get_line_starts(&contents).collect();

        Self {
            source_contents: SourceBacking::Explicit(contents),
            source_path,
            line_starts,
        }
    }

    pub fn implicit(mmap: Mmap, source_path: FileName) -> Result<Self, std::str::Utf8Error> {
        let source_contents = SourceBacking::implicit(mmap)?;
        if let SourceBacking::ImplicitMMap(yoke) = &source_contents {
            #[cfg(unix)]
            if let Err(e) = yoke.backing_cart().advise(Advice::Sequential) {
                warn!(mmap = ?yoke.backing_cart(), "Failed to change mmap behavior: {e}");
            }
        }
        let line_starts = Self::get_line_starts(source_contents.as_ref()).collect();

        Ok(Self {
            source_contents,
            source_path,
            line_starts,
        })
    }

    pub fn path(&self) -> &FileName {
        &self.source_path
    }

    pub fn contents(&self) -> &str {
        self.source_contents.as_ref()
    }

    pub(crate) fn line_starts(&self) -> &[usize] {
        &self.line_starts
    }
}

impl<'a> CodeHolder for &'a ReadSource {
    type Str = &'a str;

    #[inline]
    fn get_chunk(self, at: CodePoint) -> Self::Str {
        let contents = self.source_contents.as_ref();
        &contents[at.as_range()]
    }
}

impl ReadSource {
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

impl<'a> Files<'a> for ReadSource {
    type FileId = ();
    type Name = FileName;
    type Source = &'a str;

    fn name(&'a self, (): ()) -> Result<Self::Name, Error> {
        Ok(self.path().clone())
    }

    fn source(&'a self, (): ()) -> Result<Self::Source, Error> {
        Ok(self.source_contents.as_ref())
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

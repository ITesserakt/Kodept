use codespan_reporting::files::{line_starts, Error, Files};
use kodept_core::code_point::CodePoint;
use kodept_core::code_source::CodeSource;
use kodept_core::file_name::FileName;
use kodept_core::structure::span::CodeHolder;
use mmap_rs::Mmap;
use std::borrow::Cow;
use std::env::current_dir;
use std::io::Read;
use std::ops::Range;
use std::str::from_utf8;
use thiserror::Error;
use yoke::Yoke;

#[derive(Debug)]
enum ReadImpl {
    Explicit(String),
    Implicit(Yoke<Cow<'static, str>, Box<Mmap>>),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum ReadCodeSourceError {
    IO(#[from] std::io::Error),
    UTF8Str(#[from] std::str::Utf8Error),
    UTF8String(#[from] std::string::FromUtf8Error),
}

#[derive(Debug)]
pub struct ReadCodeSource {
    source_contents: ReadImpl,
    source_path: FileName,
    line_starts: Vec<usize>,
}

impl ReadCodeSource {
    pub fn path(&self) -> FileName {
        self.source_path.clone()
    }

    pub fn contents(&self) -> &str {
        match &self.source_contents {
            ReadImpl::Explicit(x) => x,
            ReadImpl::Implicit(x) => x.get(),
        }
    }

    pub(crate) fn line_starts(&self) -> &[usize] {
        &self.line_starts
    }
}

impl TryFrom<CodeSource> for ReadCodeSource {
    type Error = ReadCodeSourceError;

    fn try_from(value: CodeSource) -> Result<Self, Self::Error> {
        let path = value.path().get_relative_path(&current_dir()?);
        match value {
            CodeSource::Memory { contents, .. } => {
                let starts = line_starts(contents.get_ref()).collect();
                Ok(Self {
                    source_contents: ReadImpl::Explicit(contents.into_inner()),
                    source_path: path,
                    line_starts: starts,
                })
            }
            CodeSource::File { mut file, .. } => {
                let mut buf = Vec::with_capacity(1024);
                file.read_to_end(&mut buf)?;
                let buf = String::from_utf8(buf)?;
                let starts = line_starts(&buf).collect();
                Ok(Self {
                    source_contents: ReadImpl::Explicit(buf),
                    source_path: path,
                    line_starts: starts,
                })
            }
            CodeSource::MappedFile { map, .. } => {
                let buf = Yoke::try_attach_to_cart(Box::new(map.into_inner()), |it| {
                    Result::<_, ReadCodeSourceError>::Ok(Cow::Borrowed(from_utf8(it)?))
                })?;
                let contents: &Cow<_> = buf.get();
                let starts = line_starts(contents).collect();
                Ok(Self {
                    source_contents: ReadImpl::Implicit(buf),
                    source_path: path,
                    line_starts: starts,
                })
            }
        }
    }
}

impl<'a> CodeHolder for &'a ReadCodeSource {
    type Str = Cow<'a, str>;
    
    fn get_chunk(self, at: CodePoint) -> Cow<'a, str> {
        match &self.source_contents {
            ReadImpl::Explicit(x) => Cow::Borrowed(&x[at.as_range()]),
            ReadImpl::Implicit(x) => Cow::Borrowed(&x.get()[at.as_range()]),
        }
    }
}

impl<'a> Files<'a> for ReadCodeSource {
    type FileId = ();
    type Name = FileName;
    type Source = &'a str;

    fn name(&'a self, (): ()) -> Result<Self::Name, Error> {
        Ok(self.path())
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

impl ReadCodeSource {
    fn line_start(&self, line_index: usize) -> Result<usize, Error> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.line_starts().len()) {
            Ordering::Less => Ok(self
                .line_starts()
                .get(line_index)
                .cloned()
                .expect("failed despite previous check")),
            Ordering::Equal => Ok(self.contents().len()),
            Ordering::Greater => Err(Error::LineTooLarge {
                given: line_index,
                max: self.line_starts().len() - 1,
            }),
        }
    }
}

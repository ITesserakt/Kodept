use codespan_reporting::files::line_starts;
use kodept_core::code_point::CodePoint;
use kodept_core::code_source::CodeSource;
use kodept_core::file_relative::{CodePath, FileRelative};
use kodept_core::structure::span::CodeHolder;
use mmap_rs::Mmap;
use std::borrow::Cow;
use std::env::current_dir;
use std::io::Read;
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
    source_path: CodePath,
    line_starts: Vec<usize>,
}

impl ReadCodeSource {
    pub fn path(&self) -> CodePath {
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

    pub fn with_filename<T>(&self, f: impl Fn(&Self) -> T) -> FileRelative<T> {
        FileRelative {
            value: f(self),
            filepath: self.source_path.clone(),
        }
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

impl CodeHolder for ReadCodeSource {
    fn get_chunk(&self, at: CodePoint) -> Cow<str> {
        match &self.source_contents {
            ReadImpl::Explicit(x) => Cow::Borrowed(&x[at.as_range()]),
            ReadImpl::Implicit(x) => Cow::Borrowed(&x.get()[at.as_range()]),
        }
    }
}

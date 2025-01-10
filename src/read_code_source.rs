use codespan_reporting::files::line_starts;
use derive_more::{Constructor, From};
use kodept_core::code_point::CodePoint;
use kodept_core::code_source::CodeSource;
use kodept_core::structure::span::CodeHolder;
use kodept_frontend::prelude::{ReadSource, Source, TryReadCode};
use memmap2::Mmap;
use std::borrow::Cow;
use std::env::current_dir;
use std::io::Read;
use std::ops::Range;
use std::str::from_utf8;
use thiserror::Error;
use yoke::Yoke;

#[derive(Debug, From)]
pub struct SourceImpl(ReadImpl);

pub type ReadCodeSource = ReadSource<SourceImpl>;

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

impl TryReadCode<CodeSource> for SourceImpl {
    type Error = ReadCodeSourceError;

    fn try_read(value: CodeSource) -> Result<ReadSource<Self>, Self::Error> {
        let path = value.path().get_relative_path(&current_dir()?);
        let (value, starts) = match value {
            CodeSource::Memory { contents, .. } => {
                let starts = line_starts(contents.get_ref()).collect();
                (ReadImpl::Explicit(contents.into_inner()), starts)
            }
            CodeSource::File { mut file, .. } => {
                let mut buf = Vec::with_capacity(1024);
                file.read_to_end(&mut buf)?;
                let buf = String::from_utf8(buf)?;
                let starts = line_starts(&buf).collect();
                (ReadImpl::Explicit(buf), starts)
            }
            CodeSource::MappedFile { map, .. } => {
                let buf = Yoke::try_attach_to_cart(Box::new(map.into_inner()), |it| {
                    Result::<_, ReadCodeSourceError>::Ok(Cow::Borrowed(from_utf8(it)?))
                })?;
                let contents: &Cow<_> = buf.get();
                let starts = line_starts(contents).collect();
                (ReadImpl::Implicit(buf), starts)
            }
        };
        Ok(ReadSource::new(value.into(), path, starts))
    }
}

impl Source for SourceImpl {
    type Ref<'a> = &'a str;

    fn as_ref(&self) -> Self::Ref<'_> {
        match &self.0 {
            ReadImpl::Explicit(x) => x.as_str(),
            ReadImpl::Implicit(x) => x.get().as_ref(),
        }
    }

    fn len(&self) -> usize {
        match &self.0 {
            ReadImpl::Explicit(x) => x.len(),
            ReadImpl::Implicit(x) => x.get().len(),
        }
    }

    fn range(&self, value: Range<usize>) -> Self::Ref<'_> {
        &self.as_ref()[value]
    }
}

#[derive(Debug, Copy, Clone, Constructor)]
pub struct CloningCodeHolder<C: CodeHolder>(C);

impl<C: CodeHolder> CodeHolder for CloningCodeHolder<C>
where
    C::Str: Into<String>,
{
    type Str = String;

    fn get_chunk(self, at: CodePoint) -> Self::Str {
        self.0.get_chunk(at).into()
    }
}

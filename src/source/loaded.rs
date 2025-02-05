use codespan_reporting::files::line_starts;
use derive_more::{Display, Error, From};
use kodept_frontend::prelude::{ReadSource, Source, TryReadCode};
use memmap2::Mmap;
use std::borrow::Cow;
use std::env::current_dir;
use std::io::Read;
use std::ops::Range;
use std::str::from_utf8;
use yoke::Yoke;

#[derive(Debug, From)]
pub struct SourceImpl(ReadImpl);

pub type CodeSource = ReadSource<SourceImpl>;

#[derive(Debug)]
enum ReadImpl {
    Explicit(String),
    Implicit(Yoke<Cow<'static, str>, Box<Mmap>>),
}

#[derive(Debug, Error, Display, From)]
pub enum CodeSourceError {
    IO(std::io::Error),
    UTF8Str(std::str::Utf8Error),
    UTF8String(std::string::FromUtf8Error),
}

impl TryReadCode<super::unloaded::CodeSource> for SourceImpl {
    type Error = CodeSourceError;

    fn try_read(value: super::unloaded::CodeSource) -> Result<ReadSource<Self>, Self::Error> {
        let path = value.path().get_relative_path(&current_dir()?);
        let (value, starts) = match value {
            super::unloaded::CodeSource::Memory { contents, .. } => {
                let starts = line_starts(contents.get_ref()).collect();
                (ReadImpl::Explicit(contents.into_inner()), starts)
            }
            super::unloaded::CodeSource::File { mut file, .. } => {
                let mut buf = Vec::with_capacity(1024);
                file.read_to_end(&mut buf)?;
                let buf = String::from_utf8(buf)?;
                let starts = line_starts(&buf).collect();
                (ReadImpl::Explicit(buf), starts)
            }
            super::unloaded::CodeSource::MappedFile { map, .. } => {
                let buf = Yoke::try_attach_to_cart(Box::new(map.into_inner()), |it| {
                    Result::<_, CodeSourceError>::Ok(Cow::Borrowed(from_utf8(it)?))
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

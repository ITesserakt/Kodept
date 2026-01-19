use derive_more::From;
use kodept_frontend::prelude::{ReadSource, Source};
use memmap2::Mmap;
use std::borrow::Cow;
use std::ops::Range;
use std::str::{Utf8Error, from_utf8};
use yoke::Yoke;

#[derive(Debug, From)]
pub struct SourceImpl(ReadImpl);

pub type CodeSource = ReadSource<SourceImpl>;

#[derive(Debug)]
enum ReadImpl {
    Explicit(String),
    Implicit(Yoke<Cow<'static, str>, Box<Mmap>>),
}

impl SourceImpl {
    pub fn explicit(value: String) -> Self {
        Self(ReadImpl::Explicit(value))
    }

    pub fn implicit(mmap: Mmap) -> Result<Self, Utf8Error> {
        let yoke =
            Yoke::try_attach_to_cart(Box::new(mmap), |it| Ok(Cow::Borrowed(from_utf8(it)?)))?;

        Ok(Self(ReadImpl::Implicit(yoke)))
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

use std::ops::Range;
use kodept_report::files::external::{Error, Files};

mod utils;

pub use utils::ExtractReports;

pub struct Global;

impl<'a> Files<'a> for Global {
    type FileId = ();
    type Name = &'static str;
    type Source = &'static str;

    fn name(&'a self, _: Self::FileId) -> Result<Self::Name, Error> {
        Ok("<global level>")
    }

    fn source(&'a self, _: Self::FileId) -> Result<Self::Source, Error> {
        Err(Error::FileMissing)
    }

    fn line_index(&'a self, _: Self::FileId, _: usize) -> Result<usize, Error> {
        Err(Error::FileMissing)
    }

    fn line_range(&'a self, _: Self::FileId, _: usize) -> Result<Range<usize>, Error> {
        Err(Error::FileMissing)
    }
}

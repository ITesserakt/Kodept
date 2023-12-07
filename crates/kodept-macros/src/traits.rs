use crate::error::report::{Report, ReportMessage};
use derive_more::From;
use kodept_ast::traits::{Accessor, IdProducer, Linker};
use kodept_core::code_point::CodePoint;
use kodept_core::file_relative::CodePath;
use std::convert::Infallible;

#[derive(Debug, From)]
pub enum UnrecoverableError {
    Report(Report),
    Infallible(Infallible),
}

impl UnrecoverableError {
    pub fn into_report(self) -> Report {
        match self {
            UnrecoverableError::Report(x) => x,
            UnrecoverableError::Infallible(_) => unreachable!(),
        }
    }
}

pub trait Reporter: FileContextual {
    fn report_and_fail<R: Into<ReportMessage>>(
        &self,
        at: Vec<CodePoint>,
        message: R,
    ) -> Result<Infallible, UnrecoverableError> {
        Err(Report::new(&self.file_path(), at, message).into())
    }

    fn add_report<R: Into<ReportMessage>>(&mut self, at: Vec<CodePoint>, message: R) {
        self.report(Report::new(&self.file_path(), at, message))
    }

    fn report(&mut self, report: Report);
}

pub trait FileContextual {
    fn file_path(&self) -> CodePath;
}

pub trait Context<'c>: IdProducer + Linker<'c> + Accessor<'c> + Reporter {}

impl<'c, T: IdProducer + Linker<'c> + Accessor<'c> + Reporter> Context<'c> for T {}

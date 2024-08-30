use codespan_reporting::files::{Error, Files};
use codespan_reporting::term::termcolor::WriteColor;
use derive_more::{From, Unwrap};
use kodept_ast::graph::SyntaxTree;
use kodept_core::code_point::CodePoint;
use kodept_core::file_relative::CodePath;
use std::rc::Weak;

use crate::error::report::{Report, ReportMessage};
use crate::error::traits::{CodespanSettings, Reportable};

#[derive(Debug, From, Unwrap)]
pub enum UnrecoverableError {
    Report(Report),
}

impl UnrecoverableError {
    pub fn into_report(self) -> Report {
        match self {
            UnrecoverableError::Report(x) => x,
        }
    }
}

impl Reportable for UnrecoverableError {
    type FileId = ();

    fn emit<'f, W: WriteColor, F: Files<'f, FileId=Self::FileId>>(self, settings: &mut CodespanSettings<W>, source: &'f F) -> Result<(), Error> {
        self.into_report().emit(settings, source)
    }
}

pub trait Reporter: FileContextual {
    fn report_and_fail<R: Into<ReportMessage>, T>(
        &self,
        at: Vec<CodePoint>,
        message: R,
    ) -> Result<T, UnrecoverableError> {
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

pub trait Context: Reporter {
    fn tree(&self) -> Weak<SyntaxTree> {
        todo!()
    }
}

pub trait MutableContext: Context {
    fn modify_tree(
        &mut self,
        f: impl FnOnce(SyntaxTree) -> SyntaxTree,
    ) -> Result<(), ReportMessage>;
}

impl<T: Reporter> Context for T {}

use std::ops::Range;
use std::rc::Rc;

use codespan_reporting::diagnostic::Severity;
use codespan_reporting::files::{Error, Files};
use replace_with::replace_with_or_abort;

use kodept_ast::graph::SyntaxTree;
use kodept_ast::rlt_accessor::RLTAccessor;
use kodept_core::file_relative::{CodePath, FileRelative};
use kodept_macros::error::report::{Report, ReportMessage};
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::error::traits::Reportable;
use kodept_macros::traits::{FileContextual, MutableContext, Reporter};

use crate::codespan_settings::CodespanSettings;
use crate::read_code_source::ReadCodeSource;

#[derive(Debug)]
pub struct DefaultContext<'r> {
    report_collector: FileRelative<ReportCollector>,
    rlt_accessor: RLTAccessor<'r>,
    tree: Rc<SyntaxTree>,
}

impl<'r> DefaultContext<'r> {
    pub fn new(
        report_collector: FileRelative<ReportCollector>,
        rlt_accessor: RLTAccessor<'r>,
        ast: SyntaxTree,
    ) -> Self {
        Self {
            report_collector,
            rlt_accessor,
            tree: Rc::new(ast),
        }
    }
}

impl FileContextual for DefaultContext<'_> {
    fn file_path(&self) -> CodePath {
        self.report_collector.filepath.clone()
    }
}

impl Reporter for DefaultContext<'_> {
    fn report(&mut self, report: Report) {
        self.report_collector.value.report(report)
    }
}

impl DefaultContext<'_> {
    pub fn emit_diagnostics(
        self,
        settings: &mut CodespanSettings,
        source: &ReadCodeSource,
    ) {
        for report in self.report_collector.value.into_collected_reports() {
            report
                .emit(settings, source)
                .expect("Cannot emit diagnostic for file");
        }
    }

    pub fn get_links(&self) -> &RLTAccessor {
        &self.rlt_accessor
    }
    pub fn has_errors(&self) -> bool {
        self.report_collector.value.has_errors()
    }
}

impl<'a> Files<'a> for ReadCodeSource {
    type FileId = ();
    type Name = CodePath;
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

struct SharedASTError;

struct DroppedASTError;

impl From<SharedASTError> for ReportMessage {
    fn from(_: SharedASTError) -> Self {
        Self::new(
            Severity::Error,
            "IE001",
            "AST still can be accessed from some places".to_string(),
        )
    }
}

impl From<DroppedASTError> for ReportMessage {
    fn from(_: DroppedASTError) -> Self {
        Self::new(
            Severity::Bug,
            "IE002",
            "AST was dropped. Concurrent access?".to_string(),
        )
    }
}

impl MutableContext for DefaultContext<'_> {
    fn modify_tree(
        &mut self,
        f: impl FnOnce(SyntaxTree) -> SyntaxTree,
    ) -> Result<(), ReportMessage> {
        match Rc::get_mut(&mut self.tree) {
            None => Err(SharedASTError.into()),
            Some(rc) => {
                replace_with_or_abort(rc, f);
                Ok(())
            },
        }
    }
}

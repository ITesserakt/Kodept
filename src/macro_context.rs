use std::ops::Range;
use std::rc::Rc;

use codespan_reporting::files::{Error, Files};
use codespan_reporting::term::termcolor::WriteColor;
use derive_more::Constructor;
use thiserror::Error;

use kodept_ast::graph::{NodeId, SyntaxTree};
use kodept_ast::rlt_accessor::{ASTFamily, RLTAccessor, RLTFamily};
use kodept_ast::traits::{Accessor, IntoASTFamily, Linker};
use kodept_core::file_relative::{CodePath, FileRelative};
use kodept_core::ConvertibleTo;
use kodept_macros::error::report::Report;
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::traits::{FileContextual, Reporter};

use crate::codespan_settings::{CodespanSettings, ReportExt};
use crate::read_code_source::ReadCodeSource;

#[derive(Debug, Constructor)]
pub struct DefaultContext<'c> {
    report_collector: FileRelative<ReportCollector>,
    rlt_accessor: RLTAccessor<'c>,
    tree: Rc<SyntaxTree>,
}

#[derive(Debug, Error)]
#[error("Compilation failed due to produced errors")]
pub struct ErrorReported;

impl<'c> Linker<'c> for DefaultContext<'c> {
    fn link_ref<A, B>(&mut self, ast: NodeId<A>, with: B)
    where
        NodeId<A>: Into<ASTFamily>,
        B: Into<RLTFamily<'c>>,
    {
        self.rlt_accessor.save(ast, with)
    }

    fn link<A, B>(&mut self, ast: A, with: B) -> A
    where
        A: IntoASTFamily,
        B: Into<RLTFamily<'c>>,
    {
        self.rlt_accessor.save(ast.as_member(), with);
        ast
    }

    fn link_existing<A: IntoASTFamily>(&mut self, a: A, b: &impl IntoASTFamily) -> A {
        self.rlt_accessor.save_existing(&a, b);
        a
    }
}

impl<'c> Accessor<'c> for DefaultContext<'c> {
    fn access<B: 'c>(&self, ast: &impl IntoASTFamily) -> Option<&B>
    where
        RLTFamily<'c>: ConvertibleTo<&'c B>,
    {
        self.rlt_accessor.access(ast)
    }

    fn access_unknown(&self, ast: &impl IntoASTFamily) -> Option<RLTFamily> {
        self.rlt_accessor.access_unknown(ast).cloned()
    }

    fn tree(&self) -> Rc<SyntaxTree> {
        self.tree.clone()
    }
}

impl<'c> FileContextual for DefaultContext<'c> {
    fn file_path(&self) -> CodePath {
        self.report_collector.filepath.clone()
    }
}

impl<'c> Reporter for DefaultContext<'c> {
    fn report(&mut self, report: Report) {
        self.report_collector.value.report(report)
    }
}

impl<'c> DefaultContext<'c> {
    pub fn emit_diagnostics<W: WriteColor>(
        self,
        settings: &mut CodespanSettings<W>,
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

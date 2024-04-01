use std::ops::Range;
use std::rc::{Rc, Weak};

use codespan_reporting::diagnostic::Severity;
use codespan_reporting::files::{Error, Files};
use codespan_reporting::term::termcolor::WriteColor;
use derive_more::Constructor;
use thiserror::Error;

use kodept_ast::graph::{NodeId, SyntaxTree};
use kodept_ast::rlt_accessor::{ASTFamily, RLTAccessor, RLTFamily};
use kodept_ast::traits::{Accessor, IntoASTFamily, Linker};
use kodept_core::file_relative::{CodePath, FileRelative};
use kodept_core::ConvertibleToRef;
use kodept_macros::error::report::{Report, ReportMessage};
use kodept_macros::error::report_collector::ReportCollector;
use kodept_macros::traits::{FileContextual, MutableContext, Reporter};

use crate::codespan_settings::{CodespanSettings, ReportExt};
use crate::read_code_source::ReadCodeSource;

#[derive(Debug, Constructor)]
pub struct DefaultContext {
    report_collector: FileRelative<ReportCollector>,
    rlt_accessor: RLTAccessor,
    tree: Option<Rc<SyntaxTree>>,
}

#[derive(Debug, Error)]
#[error("Compilation failed due to produced errors")]
pub struct ErrorReported;

impl Linker for DefaultContext {
    fn link_ref<A, B>(&mut self, ast: NodeId<A>, with: &B)
    where
        NodeId<A>: Into<ASTFamily>,
        B: Into<RLTFamily> + Clone,
    {
        self.rlt_accessor.save(ast, with)
    }

    fn link<A, B>(&mut self, ast: A, with: &B) -> A
    where
        A: IntoASTFamily,
        B: Into<RLTFamily> + Clone,
    {
        self.rlt_accessor.save(ast.as_member(), with);
        ast
    }

    fn link_existing<A: IntoASTFamily>(&mut self, a: A, b: &impl IntoASTFamily) -> A {
        self.rlt_accessor.save_existing(&a, b);
        a
    }
}

impl Accessor for DefaultContext {
    fn access<B>(&self, ast: &impl IntoASTFamily) -> Option<&B>
    where
        RLTFamily: ConvertibleToRef<B>,
    {
        self.rlt_accessor.access(ast)
    }

    fn access_unknown(&self, ast: &impl IntoASTFamily) -> Option<RLTFamily> {
        self.rlt_accessor.access_unknown(ast).cloned()
    }

    fn tree(&self) -> Weak<SyntaxTree> {
        Rc::downgrade(&self.tree.as_ref().expect("AST was deallocated"))
    }
}

impl FileContextual for DefaultContext {
    fn file_path(&self) -> CodePath {
        self.report_collector.filepath.clone()
    }
}

impl Reporter for DefaultContext {
    fn report(&mut self, report: Report) {
        self.report_collector.value.report(report)
    }
}

impl DefaultContext {
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

impl MutableContext for DefaultContext {
    fn modify_tree(
        &mut self,
        f: impl FnOnce(SyntaxTree) -> SyntaxTree,
    ) -> Result<(), ReportMessage> {
        match self.tree.take() {
            None => Err(DroppedASTError.into()),
            Some(rc) => match Rc::try_unwrap(rc) {
                Ok(this) => {
                    self.tree = Some(Rc::new(f(this)));
                    Ok(())
                }
                Err(this) => {
                    self.tree = Some(this);
                    Err(SharedASTError.into())
                }
            },
        }
    }
}

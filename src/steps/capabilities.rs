use crate::macro_context::TreeTraversal;
use kodept_ast::graph::dfs::DetachedDfsIter;
use kodept_ast::graph::stage::FullAccess;
use kodept_ast::graph::SyntaxTree;
use kodept_ast::rlt_accessor::RLTAccessor;
use kodept_core::file_relative::CodePath;
use kodept_macros::context::{FileProvider, Reporter, SyntaxProvider};
use kodept_macros::error::report::Report;
use kodept_macros::error::report_collector::ReportCollector;
use std::ops::{Deref, DerefMut};

pub struct BasicCapability<'r> {
    pub file: CodePath,
    pub ast: SyntaxTree<FullAccess>,
    pub rlt: RLTAccessor<'r>,
    pub reporter: ReportCollector
}

impl DerefMut for BasicCapability<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ast
    }
}

impl Deref for BasicCapability<'_> {
    type Target = SyntaxTree<FullAccess>;

    fn deref(&self) -> &Self::Target {
        &self.ast
    }
}

impl SyntaxProvider for BasicCapability<'_> {
}

impl TreeTraversal for BasicCapability<'_> {
    type Access = FullAccess;

    fn detached_iter(&self) -> DetachedDfsIter {
        self.ast.dfs().detach()
    }

    fn get_tree(&self) -> &SyntaxTree<Self::Access> {
        &self.ast
    }
}

impl FileProvider for BasicCapability<'_> {
    fn path(&self) -> CodePath {
        self.file.clone()
    }
}

impl Reporter for BasicCapability<'_> {
    fn report(&mut self, report: Report) {
        self.reporter.report(report)
    }
}

use kodept::source::collection::SourceView;
use kodept_ast::syntax_tree::prelude::{SourceCode, AST};
use kodept_ast_nodes::file::FileDecl;
use kodept_core::file_name::FileDescriptor;
use kodept_core::structure::span::CodeHolder;
use kodept_rlt::prelude::RLT;
use std::borrow::Cow;

#[cfg(feature = "interning")]
pub fn build_ast(source: &SourceView, rlt: RLT) -> AST {
    let code_holder =
        kodept_interning::InterningCodeHolder::new(&**source).map(|it| Cow::Borrowed(it.0));
    let ast = AST::recursively_build::<FileDecl>(
        rlt,
        SourceCode::new(
            code_holder,
            FileDescriptor::new(source.path().clone(), *source.id),
        ),
    );
    let metrics = kodept_interning::metrics::InterningMetrics::gather();
    let (saved_value, saved_suffix) = metrics.memory_save();
    tracing::debug!(
        ?metrics,
        "Interning saved {:.2}{}",
        saved_value,
        saved_suffix
    );
    ast
}

#[cfg(not(feature = "interning"))]
pub fn build_ast(source: &SourceView, rlt: RLT) -> AST {
    let code_holder = source.map(|it| Cow::Owned(it.to_string()));
    AST::recursively_build::<FileDecl>(
        rlt,
        SourceCode::new(
            code_holder,
            FileDescriptor::new(source.path().clone(), *source.id),
        ),
    )
}

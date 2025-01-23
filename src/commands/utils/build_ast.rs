use kodept::source::collection::SourceView;
use kodept_core::structure::span::CodeHolder;
use kodept_rlt::prelude::RLT;
use std::borrow::Cow;
use kodept_ast::resource::rlt::SyntaxResolver;
use kodept_ast::syntax_tree::prelude::AST;
use kodept_ast_nodes::file::FileDecl;

#[cfg(feature = "interning")]
pub fn build_ast(source: &SourceView, rlt: RLT) -> (AST, SyntaxResolver) {
    let source =
        kodept_interning::InterningCodeHolder::new(&**source).map(|it| Cow::Borrowed(it.0));
    AST::recursively_build::<FileDecl>(rlt, source)
}

#[cfg(not(feature = "interning"))]
pub fn build_ast(source: &SourceView, rlt: RLT) -> (AST, SyntaxResolver) {
    let source = source.map(|it| Cow::Owned(it.to_string()));
    AST::recursively_build::<FileDecl>(rlt, source)
}

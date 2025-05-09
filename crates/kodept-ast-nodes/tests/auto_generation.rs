use kodept_ast::syntax_tree::prelude::{SourceCode, AST};
use kodept_ast::Str;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::code_point::CodePoint;
use kodept_core::file_name::{FileDescriptor, FileId, FileName};
use kodept_core::structure::CodeHolder;
use kodept_rlt::prelude::RLT;
use proptest::{prop_assert, proptest};

#[derive(Debug, Copy, Clone)]
struct FakeSourceCode;

impl CodeHolder for FakeSourceCode {
    type Str = Str;

    #[inline]
    fn get_chunk(self, _: CodePoint) -> Self::Str {
        Str::Borrowed("")
    }
}

proptest! {
    #[test]
    fn test_conversion_with_autogeneration(rlt: RLT) {
        let mut ast = AST::recursively_build::<FileDecl>(rlt,
            SourceCode::new(FakeSourceCode,
                FileDescriptor::new(FileName::Anon, FileId::generate())));

        prop_assert!(ast.interact().immediate_exclusive(|w| w.entities().used_count()) > 0);
    }
}

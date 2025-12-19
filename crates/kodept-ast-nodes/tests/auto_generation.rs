use bevy_ecs::prelude::World;
use kodept_ast::Str;
use kodept_ast_nodes::file::FileDecl;
use kodept_core::code_point::CodePoint;
use kodept_core::structure::CodeHolder;
use kodept_rlt::prelude::RLT;
use proptest::{prop_assert, proptest};

#[derive(Debug, Copy, Clone)]
struct FakeSourceCode;

impl CodeHolder for FakeSourceCode {
    type Str = Str;

    #[inline]
    fn get_chunk(self, _: CodePoint) -> Self::Str {
        // emulate string literals, because this is the only way to bypass error with literals parsing
        Str::Borrowed("\"test\"")
    }
}

proptest! {
    #[test]
    fn test_conversion_with_autogeneration(rlt: RLT) {
        let mut world = World::new();
        let result = FileDecl::from_syntax(&rlt.0, FakeSourceCode, world.commands());

        prop_assert!(result.is_ok(), "Expected success build, but encountered an error: {:?}", result);
    }
}

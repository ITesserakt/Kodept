use bevy_ecs::prelude::World;
use kodept_ast::prelude::NodeId;
use kodept_ast::syntax_tree::experimental::GenericSpawnContext;
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

    #[test]
    fn test_conversion_with_autogeneration_v2(rlt: RLT) {
        let mut world = World::new();
        for module in rlt.0.0 {
            let result: Result<NodeId<kodept_ast_nodes::v3::module::Module>, _> = GenericSpawnContext::top_level(&module, world.commands(), FakeSourceCode);
            prop_assert!(result.is_ok(), "Expected success build, but encountered an error: {:?}", result);
        }
    }
}

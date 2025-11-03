use kodept_ast::resource::reflection::DebugRegistry;
use kodept_frontend::engine::{Engine, Plugin};

pub struct RegisterReflectionPlugin;

impl Plugin for RegisterReflectionPlugin {
    fn build(self, engine: &mut Engine) {
        let mut registry = DebugRegistry::new();
        kodept_ast_nodes::register_reflection_info(&mut registry);
        kodept_ast::register_reflection_info(&mut registry);
        
        engine.insert_resource(registry);
    }
}
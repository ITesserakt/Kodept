use crate::Cli;
use kodept_ast::resource::reflection::DebugRegistry;
use kodept_cli::prelude::{LogPlugin, LoggingLevel, ReportsPlugin};
use kodept_frontend::engine::utils::TaskPoolPlugin;
use kodept_frontend::engine::{Engine, Plugin};

struct RegisterReflectionPlugin;

impl Plugin for RegisterReflectionPlugin {
    fn build(self, engine: &mut Engine) {
        let mut registry = DebugRegistry::new();
        kodept_ast_nodes::register_reflection_info(&mut registry);
        kodept_ast::register_reflection_info(&mut registry);

        engine.insert_resource(registry);
    }
}

pub(crate) struct Plugins<'a> {
    pub(crate) config: &'a Cli,
}

impl Plugin for Plugins<'_> {
    fn build(self, engine: &mut Engine) {
        engine
            .add_plugin(RegisterReflectionPlugin)
            .add_plugin(LogPlugin {
                level: LoggingLevel::Debug,
                display_thread_names: false,
            })
            .add_plugin_if(
                !self.config.diagnostic_config.disable,
                ReportsPlugin {
                    config: &self.config.diagnostic_config,
                },
            )
            .add_plugin(TaskPoolPlugin::default());
    }
}

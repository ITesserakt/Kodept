use kodept_frontend::engine::{Engine, Plugin};
use kodept_frontend::engine::reporter;
use tracing::Level;
use kodept_report::codespan::external::{ColorChoice, Config};
use kodept_report::prelude::CodespanSettings;

#[derive(Debug)]
pub struct LogPlugin {
    pub level: Level,
    pub display_thread_names: bool,
}

impl Default for LogPlugin {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            display_thread_names: true,
        }
    }
}

impl Plugin for LogPlugin {
    fn build(self, _: &mut Engine) {
        tracing_subscriber::fmt()
            .with_thread_names(self.display_thread_names)
            .with_max_level(self.level)
            .init()
    }
}

#[derive(Debug)]
pub struct ThreadPoolPlugin {
    pub total_threads: usize,
}

impl Default for ThreadPoolPlugin {
    fn default() -> Self {
        Self { total_threads: 1 }
    }
}

impl Plugin for ThreadPoolPlugin {
    fn build(self, engine: &mut Engine) {
        let max_total_threads = (self.total_threads / 2).max(1);
        engine.add_plugin(kodept_frontend::engine::utils::TaskPoolPlugin {
            task_pool_options: kodept_frontend::engine::utils::TaskPoolOptions {
                max_total_threads,
                min_total_threads: 1,
                ..Default::default()
            },
        });
        // TODO: combine bevy's thread pool with rayon's one
        //       Maybe `Forte`? (https://github.com/NthTensor/Forte)
        #[cfg(feature = "parallel")]
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.total_threads / 2)
            .build_global()
            .expect("Cannot initialize rayon thread pool");
    }
}

pub struct ReportsPlugin {
    pub eager: bool,
    pub colored: ColorChoice,
    pub config: Config
}

impl Default for ReportsPlugin {
    fn default() -> Self {
        Self {
            eager: true,
            colored: ColorChoice::Auto,
            config: Config::default()
        }
    }
}

impl Plugin for ReportsPlugin {
    fn build(self, engine: &mut Engine) {
        let config = CodespanSettings::stderr(self.config, self.colored);
        let settings = match self.eager {
            true => reporter::Settings::Eager(config),
            false => reporter::Settings::Lazy(config),
        };
        engine.insert_resource(settings);
    }
}

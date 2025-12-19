use tracing::Level;
use kodept_frontend::engine::{reporter, Engine, Plugin};
use kodept_report::codespan::external::Config;
use kodept_report::prelude::CodespanSettings;
use crate::prelude::{LoggingLevel, DiagnosticConfig, DisplayStyle, ColorChoice};

#[derive(Debug)]
pub struct LogPlugin {
    pub level: LoggingLevel,
    pub display_thread_names: bool,
}

impl Default for LogPlugin {
    fn default() -> Self {
        Self {
            level: LoggingLevel::Info,
            display_thread_names: true,
        }
    }
}

impl Plugin for LogPlugin {
    fn build(self, _: &mut Engine) {
        tracing_subscriber::fmt()
            .with_thread_names(self.display_thread_names)
            .with_max_level(match self.level {
                LoggingLevel::Trace => Level::TRACE,
                LoggingLevel::Debug => Level::DEBUG,
                LoggingLevel::Info => Level::INFO,
                LoggingLevel::Warning => Level::WARN,
                LoggingLevel::Error => Level::ERROR,
            })
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
    pub config: DiagnosticConfig
}

impl Plugin for ReportsPlugin {
    fn build(self, engine: &mut Engine) {
        let mut config = Config {
            display_style: match self.config.style {
                DisplayStyle::Rich => kodept_report::codespan::external::DisplayStyle::Rich,
                DisplayStyle::Medium => kodept_report::codespan::external::DisplayStyle::Medium,
                DisplayStyle::Short => kodept_report::codespan::external::DisplayStyle::Short,
            },
            tab_width: self.config.tab_width,
            ..Default::default()
        };
        if self.config.show_full_context_lines {
            config.start_context_lines = usize::MAX;
            config.end_context_lines = usize::MAX;
        }

        let settings = CodespanSettings::stderr(
            config,
            match self.config.color {
                ColorChoice::Auto => kodept_report::codespan::external::ColorChoice::Auto,
                ColorChoice::Always => kodept_report::codespan::external::ColorChoice::Always,
                ColorChoice::Never => kodept_report::codespan::external::ColorChoice::Never,
            },
        );
        let settings = match self.config.eager {
            true => reporter::Settings::Eager(settings),
            false => reporter::Settings::Lazy(settings),
        };
        engine.insert_resource(settings);
    }
}

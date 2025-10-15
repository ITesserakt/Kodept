use crate::cli::configs::DiagnosticConfig;
use kodept_frontend::engine::Engine;
use kodept_frontend::engine::reporter::Settings;
use kodept_report::codespan::CodespanSettings;
use kodept_report::codespan::external::Config;

pub mod configs;
pub mod primary;
pub mod utils;

pub fn init_reports(value: DiagnosticConfig) -> impl FnOnce(&mut Engine) {
    let mut config = Config {
        display_style: value.style.into(),
        tab_width: value.tab_width,
        ..Default::default()
    };
    if value.show_full_context_lines {
        config.start_context_lines = usize::MAX;
        config.end_context_lines = usize::MAX;
    }
    let settings = match (value.disable, value.eager) {
        (true, _) => Settings::Disabled,
        (false, true) => Settings::Eager(CodespanSettings::stderr(config, value.color)),
        (false, false) => Settings::Lazy(CodespanSettings::stderr(config, value.color)),
    };

    move |engine| {
        engine.insert_resource(settings);
    }
}

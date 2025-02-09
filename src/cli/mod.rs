use crate::cli::configs::DiagnosticConfig;
use kodept::report::GlobalReports;
use kodept_report::codespan::CodespanSettings;
use kodept_report::codespan::external::Config;

pub mod configs;
pub mod primary;
pub mod utils;

pub fn make_reports(value: DiagnosticConfig) -> GlobalReports {
    let config = Config {
        display_style: value.style.into(),
        tab_width: value.tab_width,
        ..Default::default()
    };
    if value.disable {
        return GlobalReports::disabled()
    }
    match value.eager {
        true => GlobalReports::eager(CodespanSettings::stderr(config, value.color)),
        false => GlobalReports::lazy(CodespanSettings::stderr(config, value.color)),
    }
}

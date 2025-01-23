use crate::cli::configs::DiagnosticConfig;
use codespan_reporting::term::termcolor::StandardStream;
use codespan_reporting::term::Config;
use kodept::report::GlobalReports;
use kodept_report::error::traits::CodespanSettings;

pub mod configs;
pub mod primary;
pub mod utils;

pub fn make_reports(value: DiagnosticConfig) -> GlobalReports {
    let config = Config {
        display_style: value.style.into(),
        tab_width: value.tab_width,
        ..Default::default()
    };
    let stream = match value.disable {
        true => return GlobalReports::disabled(),
        false => StandardStream::stderr(value.color.0),
    };
    match value.eager {
        true => GlobalReports::eager(CodespanSettings { config, stream }),
        false => GlobalReports::lazy(CodespanSettings { config, stream }),
    }
}

use crate::cli::configs::LoadingConfig;
use kodept::loader::Loader;
use kodept::report::{GlobalReports, Reports};
use kodept::source::collection::Sources;
use kodept::source::load_each_source;
use kodept_frontend::Execution;
use std::ops::ControlFlow::{Break, Continue};
use std::sync::Arc;

pub fn get_all_sources(
    config: &LoadingConfig,
    reports: GlobalReports,
) -> Execution<(Arc<Sources>, Reports)> {
    let loader = match Loader::try_from(config) {
        Ok(x) => x,
        Err(e) => return Break(reports.report(e)?),
    };
    match load_each_source(loader) {
        Ok(x) => {
            let sources = Arc::new(x);
            Continue((sources.clone(), reports.upgrade(sources)))
        }
        Err(e) => Break(reports.report(e)?),
    }
}

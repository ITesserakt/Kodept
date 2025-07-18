use crate::cli::configs::LoadingConfig;
use kodept::loader::Loader;
use kodept::report::{GlobalReports, Reports};
use kodept::source::collection::Sources;
use kodept::source::load_each_source;
use kodept_frontend::prelude::ExtractReports;
use kodept_frontend::Execution;
use std::ops::ControlFlow::Continue;
use std::sync::Arc;

pub fn get_all_sources(
    config: &LoadingConfig,
    reports: GlobalReports,
) -> Execution<(Arc<Sources>, Reports)> {
    let loader = Loader::try_from(config).extract_reports_global(&reports)?;
    let sources = Arc::new(load_each_source(loader).extract_reports_global(&reports)?);
    Continue((sources.clone(), reports.upgrade(sources)))
}

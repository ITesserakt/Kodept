use kodept::codespan_settings::CodespanSettings;
use kodept::common_iter::CommonIter;
use kodept::read_code_source::ReadCodeSource;
use kodept_macros::error::ErrorReported;
use std::panic::{RefUnwindSafe, UnwindSafe};

pub trait Command {
    type Params: UnwindSafe;

    fn exec(
        &self,
        sources: impl CommonIter<Item = ReadCodeSource> + UnwindSafe,
        settings: CodespanSettings,
        additional_params: Self::Params,
    ) -> Result<(), ErrorReported>
    where
        Self::Params: Clone + Send,
        Self: Sync + RefUnwindSafe,
    {
        match std::panic::catch_unwind(move || {
            sources.try_foreach_with(
                (settings, additional_params),
                |(settings, params), source| self.exec_for_source(source, settings, params),
            )
        }) {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(ErrorReported::new())
        }
    }

    fn exec_for_source(
        &self,
        source: ReadCodeSource,
        settings: &mut CodespanSettings,
        params: &mut Self::Params,
    ) -> Result<(), ErrorReported>;
}

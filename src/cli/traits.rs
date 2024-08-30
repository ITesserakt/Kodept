use kodept::codespan_settings::CodespanSettings;
use kodept::read_code_source::ReadCodeSource;
use kodept_macros::error::ErrorReported;
use std::panic::{RefUnwindSafe, UnwindSafe};

#[cfg(feature = "parallel")]
pub trait CommonIter: rayon::prelude::ParallelIterator<Item = <Self as CommonIter>::Item> {
    type Item;
}

#[cfg(not(feature = "parallel"))]
pub trait CommonIter: Iterator<Item = <Self as CommonIter>::Item> {
    type Item;
}

#[cfg(not(feature = "parallel"))]
impl<T: Iterator> CommonIter for T {
    type Item = T::Item;
}

#[cfg(feature = "parallel")]
impl<T: rayon::prelude::ParallelIterator> CommonIter for T {
    type Item = T::Item;
}

pub trait Command {
    type Params: UnwindSafe;

    #[allow(unused_mut)]
    fn exec(
        &self,
        sources: impl CommonIter<Item = ReadCodeSource> + UnwindSafe,
        mut settings: CodespanSettings,
        mut additional_params: Self::Params,
    ) -> Result<(), ErrorReported>
    where
        Self::Params: Clone + Send,
        Self: Sync + RefUnwindSafe,
    {
        match std::panic::catch_unwind(move || {
            #[cfg(feature = "parallel")]
            {
                sources.try_for_each_with(
                    (settings, additional_params),
                    |(settings, params), source| self.exec_for_source(source, settings, params),
                )
            }
            #[cfg(not(feature = "parallel"))]
            {
                for source in sources {
                    self.exec_for_source(source, &mut settings, &mut additional_params)?;
                }
                Ok(())
            }
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

use kodept::codespan_settings::CodespanSettings;
use kodept::read_code_source::ReadCodeSource;
use kodept_macros::error::ErrorReported;
use std::panic::{RefUnwindSafe, UnwindSafe};

pub trait CommonIter {
    type Item;

    fn try_foreach_with<T, E, F>(
        self,
        with: T,
        f: F,
    ) -> Result<(), E>
    where
        T: Send + Clone,
        F: Fn(&mut T, Self::Item) -> Result<(), E>,
        F: Send + Sync,
        E: Send;
}

#[cfg(not(feature = "parallel"))]
impl<I: Iterator> CommonIter for I {
    type Item = I::Item;

    fn try_foreach_with<T, E, F>(self, mut with: T, f: F) -> Result<(), E>
    where
        F: Fn(&mut T, Self::Item) -> Result<(), E>
    {
        for item in self {
            f(&mut with, item)?;
        }
        Ok(())
    }
}

#[cfg(feature = "parallel")]
impl<I: rayon::prelude::ParallelIterator> CommonIter for I {
    type Item = I::Item;

    fn try_foreach_with<T, E, F>(self, with: T, f: F) -> Result<(), E>
    where
        T: Send + Clone,
        F: Fn(&mut T, Self::Item) -> Result<(), E>,
        F: Send + Sync,
        E: Send
    {
        rayon::prelude::ParallelIterator::try_for_each_with(self, with, f)
    }
}

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

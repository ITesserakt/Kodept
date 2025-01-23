use bevy_ecs::prelude::IntoSystemConfigs;
use bevy_ecs::schedule::SystemConfigs;
use bevy_ecs::system::Local;

pub mod frontend;
pub mod plugin;
mod read_code_source;
mod source_files;

pub mod prelude {
    pub use super::read_code_source::{ReadSource, Source, TryReadCode};
    pub use super::source_files::{SourceFiles, SourceView};
}

pub mod external {
    pub use bevy_ecs::prelude::{Component, Resource};
}

pub trait SystemExt<M> {
    fn run_once(self) -> SystemConfigs;
}

impl<M, T: IntoSystemConfigs<M>> SystemExt<M> for T {
    fn run_once(self) -> SystemConfigs {
        self.run_if(|mut lock: Local<bool>| match *lock {
            true => false,
            false => {
                *lock = true;
                true
            }
        })
    }
}

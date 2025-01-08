use tracing::warn;

pub mod context;
pub mod error;

pub mod execution {
    #[deprecated]
    pub type Execution<E, R = ()> = Result<R, E>;
}

pub fn warn_about_broken_rlt<T>() {
    warn!(
        expected = std::any::type_name::<T>(),
        "Skipping some checks because node in RLT either doesn't exist or has different type."
    );
}

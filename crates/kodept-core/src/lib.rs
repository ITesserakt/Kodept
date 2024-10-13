use std::ops::Deref;

pub mod code_point;
pub mod code_source;
pub mod file_name;
pub mod structure;

pub mod macros {
    #[macro_export]
    macro_rules! static_assert_size {
        ($ty:ty, $size:expr) => {
            const _: [(); $size] = [(); std::mem::size_of::<$ty>()];
        };
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Freeze<T>(T);

impl<T> Deref for Freeze<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Freeze<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

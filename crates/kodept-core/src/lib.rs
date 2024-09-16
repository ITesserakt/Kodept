use std::ops::Deref;

pub mod code_point;
pub mod code_source;
pub mod file_name;
pub mod structure;

pub trait ConvertibleToRef<Output> {
    fn try_as_ref(&self) -> Option<&Output>;
}

pub trait ConvertibleToMut<Output>: ConvertibleToRef<Output> {
    fn try_as_mut(&mut self) -> Option<&mut Output>;
}

impl<T, U> ConvertibleToRef<U> for T
where
    for<'a> &'a U: TryFrom<&'a T>,
{
    fn try_as_ref(&self) -> Option<&U> {
        self.try_into().ok()
    }
}

impl<T, U> ConvertibleToMut<U> for T
where
    for<'a> &'a mut U: TryFrom<&'a mut T>,
    T: ConvertibleToRef<U>,
{
    fn try_as_mut(&mut self) -> Option<&mut U> {
        self.try_into().ok()
    }
}

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

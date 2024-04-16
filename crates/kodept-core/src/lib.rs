pub mod code_point;
pub mod code_source;
pub mod file_relative;
pub mod loader;
pub mod structure;

pub trait ConvertibleToRef<Output> {
    fn try_as_ref(&self) -> Option<&Output>;
}

pub trait ConvertibleToMut<Output>: ConvertibleToRef<Output> {
    fn try_as_mut(&mut self) -> Option<&mut Output>;
}

pub trait ConvertibleTo<Output> {
    fn try_as(self) -> Option<Output>;
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

impl<T, U> ConvertibleTo<U> for T
where
    U: TryFrom<T>,
{
    fn try_as(self) -> Option<U> {
        self.try_into().ok()
    }
}

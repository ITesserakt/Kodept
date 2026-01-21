use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NEVec<T> {
    pub head: T,
    pub tail: Vec<T>,
}

pub struct NEVecIter<'a, T> {
    head: std::iter::Once<&'a T>,
    tail: std::slice::Iter<'a, T>,
}

#[macro_export]
macro_rules! nev {
    ($head:expr, $($tail:expr$(,)?)+) => {
        $crate::qbe::utils::NEVec {
            head: $head,
            tail: vec![$($tail)+]
        }
    };
    ($head:expr) => {
        $crate::qbe::utils::NEVec {
            head: $head,
            tail: Vec::new()
        }
    }
}

pub(crate) struct JoinedDisplay<'a, T>(T, &'a str);

pub(crate) trait JoinExt: Sized {
    fn join(self, separator: &str) -> JoinedDisplay<'_, Self>;
}

impl<T: Sized> JoinExt for T {
    fn join(self, separator: &str) -> JoinedDisplay<'_, Self> {
        JoinedDisplay(self, separator)
    }
}

impl<'a, 'b, T> Display for JoinedDisplay<'a, &'b T>
where
    &'b T: IntoIterator<Item: Display>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for item in self.0.into_iter() {
            if first {
                first = false;
                write!(f, "{item}")?;
            } else {
                write!(f, "{}{item}", self.1)?;
            }
        }
        Ok(())
    }
}

impl<'a, T> IntoIterator for &'a NEVec<T> {
    type Item = &'a T;
    type IntoIter = NEVecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        NEVecIter {
            head: std::iter::once(&self.head),
            tail: self.tail.iter(),
        }
    }
}

impl<'a, T> Iterator for NEVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.head.next().or_else(|| self.tail.next())
    }
}

use crate::assumption::RawAssumptionSet::Map;
use crate::r#type::MonomorphicType;
use crate::utils::JoinedDisplay;
use RawAssumptionSet::{Empty, Single};
use kodept_interning::{InternInto, Interned};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub trait TypeTable<Name>: Sized {
    type Item<'a>;

    fn resolve(&self, name: Name) -> Self::Item<'_>;
    fn merge(&mut self, other: Self);
    fn empty() -> Self;
    fn single(key: Name, value: Self::Item<'_>) -> Self;

    fn merge_many(iter: impl IntoIterator<Item = Self>) -> Self {
        iter.into_iter().fold(Self::empty(), |mut a, b| {
            a.merge(b);
            a
        })
    }

    fn push(&mut self, key: Name, value: Self::Item<'_>) {
        self.merge(Self::single(key, value));
    }
}

#[derive(Clone)]
enum RawAssumptionSet<Name> {
    Empty,
    Single(Name, Vec<Interned<MonomorphicType>>),
    Map(HashMap<Name, Vec<Interned<MonomorphicType>>>),
}

#[derive(Clone)]
pub struct AssumptionSet<Name>(RawAssumptionSet<Name>);

impl<Name> TypeTable<Name> for AssumptionSet<Name>
where
    Name: Hash + Eq,
{
    type Item<'a> = Cow<'a, [Interned<MonomorphicType>]>;

    fn resolve(&self, name: Name) -> Self::Item<'_> {
        match &self.0 {
            Empty => Cow::Borrowed(&[]),
            Single(n, x) if n == &name => Cow::Borrowed(x),
            Single(_, _) => Cow::Borrowed(&[]),
            Map(x) => match x.get(&name) {
                None => Cow::Borrowed(&[]),
                Some(x) => Cow::Borrowed(x),
            },
        }
    }

    fn merge(&mut self, other: Self) {
        match (&mut self.0, other.0) {
            (_, Empty) => {}
            (Empty, b) => self.0 = b,
            (Single(n1, x1), Single(n2, x2)) if n1 == &n2 => x1.extend(x2),
            (Single(_, _), Single(n2, x2)) => {
                let Single(n1, x1) = std::mem::replace(&mut self.0, Empty) else {
                    unreachable!()
                };
                self.0 = Map(HashMap::from([(n1, x1), (n2, x2)]))
            }
            (Map(m), Single(n, x)) => m.entry(n).or_default().extend(x),
            (Single(_, _), Map(mut m)) => {
                let Single(n, x) = std::mem::replace(&mut self.0, Empty) else {
                    unreachable!()
                };
                m.entry(n).or_default().extend(x);
                self.0 = Map(m)
            }
            (Map(m1), Map(m2)) => {
                for (k, v) in m2 {
                    m1.entry(k).or_default().extend(v)
                }
            }
        }
    }

    fn empty() -> Self {
        Self(Empty)
    }

    fn single(key: Name, value: Self::Item<'_>) -> Self {
        Self(Single(key, value.into_owned()))
    }
}

impl<Name> AssumptionSet<Name> {
    pub fn resolve_take(&mut self, key: Name) -> Cow<'_, [Interned<MonomorphicType>]>
    where
        Name: Eq + Hash,
    {
        match &mut self.0 {
            Empty => Cow::Borrowed(&[]),
            Single(k, v) if k == &key => {
                let result = std::mem::replace(v, vec![]);
                self.0 = Empty;
                Cow::Owned(result)
            }
            Single(_, _) => Cow::Borrowed(&[]),
            Map(x) => match x.remove(&key) {
                Some(v) => Cow::Owned(v),
                None => Cow::Borrowed(&[]),
            },
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = (Name, Vec<Interned<MonomorphicType>>)> {
        enum Either<A, B> {
            Left(A),
            Right(B),
        }

        impl<T, A, B> Iterator for Either<A, B>
        where
            A: Iterator<Item = T>,
            B: Iterator<Item = T>,
        {
            type Item = T;

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Either::Left(x) => x.next(),
                    Either::Right(x) => x.next(),
                }
            }
        }

        match self.0 {
            Empty => Either::Left(Either::Left(std::iter::empty())),
            Single(k, v) => Either::Left(Either::Right(std::iter::once((k, v)))),
            Map(map) => Either::Right(map.into_iter()),
        }
    }

    pub fn push_single(&mut self, name: Name, value: impl InternInto<MonomorphicType>)
    where
        Name: Hash + Eq,
    {
        self.push(name, Cow::Borrowed(&[value.intern_into()]))
    }
}

impl<Name: Display> Display for AssumptionSet<Name> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Empty => write!(f, "[]"),
            Single(k, v) => write!(f, "[{k} :: [{}]]", JoinedDisplay::enumerate(v)),
            Map(map) => {
                write!(
                    f,
                    "[{}]",
                    JoinedDisplay::enumerate(map.iter().map(|(key, value)| format!(
                        "{key} :: [{}]",
                        JoinedDisplay::enumerate(value)
                    )))
                    .join()
                )
            }
        }
    }
}

impl<Name: Debug> Debug for AssumptionSet<Name> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Empty => write!(f, "[]"),
            Single(k, v) => write!(f, "[{k:?} :: [{}]]", JoinedDisplay::enumerate(v)),
            Map(map) => {
                write!(
                    f,
                    "[{}]",
                    JoinedDisplay::enumerate(map.iter().map(|(key, value)| format!(
                        "{key:?} :: [{}]",
                        JoinedDisplay::enumerate(value)
                    )))
                    .join()
                )
            }
        }
    }
}

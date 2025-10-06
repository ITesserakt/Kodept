use crate::assumption::RawAssumptionSet::Map;
use crate::r#type::MonomorphicType;
use RawAssumptionSet::{Empty, Single};
use kodept_interning::Interned;
use std::borrow::Cow;
use std::collections::HashMap;
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

enum RawAssumptionSet<Name> {
    Empty,
    Single(Name, Vec<Interned<MonomorphicType>>),
    Map(HashMap<Name, Vec<Interned<MonomorphicType>>>),
}

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

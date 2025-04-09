use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::num::NonZeroU8;
use std::ops::BitAnd;

use derive_more::{Constructor, From};
use itertools::{concat, Itertools};
use nonempty_collections::{IntoNonEmptyIterator, NEVec, NonEmptyIterator};

use crate::substitution::Substitutions;
use crate::traits::{FreeTypeVars, Substitutable};
use crate::InferState;

#[allow(dead_code)]
fn expand_to_string(id: usize, alphabet: &'static str) -> String {
    if id == 0 {
        return alphabet
            .chars()
            .next()
            .expect("Alphabet should contain at least one letter")
            .to_string();
    }

    let alphabet: Vec<_> = alphabet.chars().collect();
    let mut current = id;
    let mut result = String::new();
    while current > 0 {
        result.push(alphabet[current % alphabet.len()]);
        current /= alphabet.len();
    }
    result
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    /// Type with two possibilities: true & false
    Boolean,
    // To keep future compatibility with hvm, builtin types should have 24 bytes size at max
    /// Parameter represents the amount of bytes for number
    I(NonZeroU8),
    /// Parameter represents the amount of bytes for number
    U(NonZeroU8),
    F24,
    /// String with some value that is known at compile time
    StaticString(u64),
    /// Array of some type which size is known at compile time
    StaticArray(u64, Box<PrimitiveType>),
}

#[derive(Copy, Clone, PartialEq, Hash, Eq, From)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct TVar(pub(crate) usize);

#[derive(Clone, PartialEq, Eq, Hash, Constructor)]
pub struct Tuple(pub(crate) Vec<MonomorphicType>);

#[derive(Clone, PartialEq, From, Eq, Hash)]
pub enum MonomorphicType {
    Primitive(PrimitiveType),
    Var(TVar),
    #[from(ignore)]
    Fn(Box<MonomorphicType>, Box<MonomorphicType>),
    Tuple(Tuple),
    Pointer(Box<MonomorphicType>),
    Constant(String),
}

#[derive(Clone, PartialEq)]
pub struct PolymorphicType {
    pub(crate) bindings: Vec<TVar>,
    pub(crate) binding_type: MonomorphicType,
}

impl PrimitiveType {
    pub const fn bool() -> Self {
        Self::Boolean
    }

    pub fn i8() -> Self {
        Self::I(NonZeroU8::new(8).unwrap())
    }

    pub fn u8() -> Self {
        Self::U(NonZeroU8::new(8).unwrap())
    }

    pub const fn custom(sign: bool, bits: NonZeroU8) -> Self {
        match sign {
            true => Self::I(bits),
            false => Self::U(bits),
        }
    }

    pub const fn f24() -> Self {
        Self::F24
    }

    pub const fn string(size: u64) -> Self {
        Self::StaticString(size)
    }

    pub fn array(count: u64, inner_type: Self) -> Self {
        Self::StaticArray(count, Box::new(inner_type))
    }
}

pub fn fun1<M: Into<MonomorphicType>, N: Into<MonomorphicType>>(
    input: N,
    output: M,
) -> MonomorphicType {
    MonomorphicType::Fn(Box::new(input.into()), Box::new(output.into()))
}

pub fn fun<M: Into<MonomorphicType>>(input: NEVec<MonomorphicType>, output: M) -> MonomorphicType {
    let (head, tail) = input.into_nonempty_iter().next();
    match (head, tail.as_slice()) {
        (x, []) => fun1(x, output),
        (x, [xs @ .., last]) => fun1(
            x,
            xs.iter().fold(fun1(last.clone(), output), |acc, next| {
                fun1(next.clone(), acc)
            }),
        ),
    }
}

pub fn var<V: Into<TVar>>(id: V) -> MonomorphicType {
    MonomorphicType::Var(id.into())
}

pub fn unit_type() -> MonomorphicType {
    MonomorphicType::Tuple(Tuple(vec![]))
}

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveType::Boolean => write!(f, "boolean"),
            PrimitiveType::I(size) => write!(f, "i{size}"),
            PrimitiveType::U(size) => write!(f, "u{size}"),
            PrimitiveType::F24 => write!(f, "f24"),
            PrimitiveType::StaticString(size) => write!(f, "s{size}"),
            PrimitiveType::StaticArray(count, ty) => write!(f, "{ty}[{count}]"),
        }
    }
}

impl Debug for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl MonomorphicType {
    fn rename(self, old: usize, new: usize) -> Self {
        match self {
            MonomorphicType::Var(TVar(id)) if id == old => TVar(new).into(),
            MonomorphicType::Primitive(_)
            | MonomorphicType::Var(_)
            | MonomorphicType::Constant(_) => self,
            MonomorphicType::Fn(input, output) => MonomorphicType::Fn(
                Box::new(input.rename(old, new)),
                Box::new(output.rename(old, new)),
            ),
            MonomorphicType::Tuple(Tuple(vec)) => MonomorphicType::Tuple(Tuple(
                vec.into_iter().map(|it| it.rename(old, new)).collect(),
            )),
            MonomorphicType::Pointer(t) => MonomorphicType::Pointer(Box::new(t.rename(old, new))),
        }
    }

    fn extract_vars(&self) -> Vec<usize> {
        match self {
            MonomorphicType::Primitive(_) | MonomorphicType::Constant(_) => vec![],
            MonomorphicType::Var(TVar(x)) => vec![*x],
            MonomorphicType::Fn(input, output) => {
                concat([input.extract_vars(), output.extract_vars()])
            }
            MonomorphicType::Tuple(Tuple(vec)) => {
                vec.iter().flat_map(MonomorphicType::extract_vars).collect()
            }
            MonomorphicType::Pointer(t) => t.extract_vars(),
        }
    }

    pub fn generalize(&self, free: &HashSet<TVar>) -> PolymorphicType {
        let diff: Vec<_> = self.free_types().difference(free).copied().collect();
        PolymorphicType {
            bindings: diff,
            binding_type: self.clone(),
        }
    }

    pub fn normalize(self) -> PolymorphicType {
        self.generalize(&HashSet::new()).normalize()
    }
}

impl PolymorphicType {
    pub(crate) fn normalize(self) -> Self {
        let mut free = self.binding_type.extract_vars();
        free.sort_unstable();
        free.dedup();
        let len = free.len();
        let binding_type = free
            .iter()
            .zip(0usize..)
            .fold(self.binding_type, |acc, (&old, new)| acc.rename(old, new));
        let bindings = free
            .into_iter()
            .zip(0usize..len)
            .map(|it| TVar(it.1))
            .collect();
        Self {
            bindings,
            binding_type,
        }
    }

    pub(crate) fn instantiate(&self, env: &mut InferState) -> MonomorphicType {
        let fresh = self.bindings.iter().map(|it| (*it, env.new_var()));
        let s0 = Substitutions::from_iter(fresh);
        self.binding_type.substitute(&s0)
    }
}

impl<S: Into<MonomorphicType>> From<S> for PolymorphicType {
    fn from(value: S) -> Self {
        Self {
            bindings: vec![],
            binding_type: value.into(),
        }
    }
}

impl BitAnd<Substitutions> for PolymorphicType {
    type Output = PolymorphicType;

    fn bitand(self, rhs: Substitutions) -> Self::Output {
        self.substitute(&rhs)
    }
}

impl BitAnd<&Substitutions> for PolymorphicType {
    type Output = PolymorphicType;

    fn bitand(self, rhs: &Substitutions) -> Self::Output {
        self.substitute(rhs)
    }
}

impl BitAnd<Substitutions> for &PolymorphicType {
    type Output = PolymorphicType;

    fn bitand(self, rhs: Substitutions) -> Self::Output {
        self.substitute(&rhs)
    }
}

impl BitAnd<&Substitutions> for &PolymorphicType {
    type Output = PolymorphicType;

    fn bitand(self, rhs: &Substitutions) -> Self::Output {
        self.substitute(rhs)
    }
}

impl BitAnd<Substitutions> for MonomorphicType {
    type Output = MonomorphicType;

    fn bitand(self, rhs: Substitutions) -> Self::Output {
        self.substitute(&rhs)
    }
}

impl BitAnd<&Substitutions> for MonomorphicType {
    type Output = MonomorphicType;

    fn bitand(self, rhs: &Substitutions) -> Self::Output {
        self.substitute(rhs)
    }
}

impl BitAnd<Substitutions> for &MonomorphicType {
    type Output = MonomorphicType;

    fn bitand(self, rhs: Substitutions) -> Self::Output {
        self.substitute(&rhs)
    }
}

impl BitAnd<&Substitutions> for &MonomorphicType {
    type Output = MonomorphicType;

    fn bitand(self, rhs: &Substitutions) -> Self::Output {
        self.substitute(rhs)
    }
}

impl Tuple {
    #[must_use]
    pub const fn unit() -> Tuple {
        Tuple(vec![])
    }

    pub fn push(&mut self, value: MonomorphicType) {
        self.0.push(value);
    }
}

impl Display for TVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "τ{}", self.0)
    }
}

impl Display for MonomorphicType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MonomorphicType::Primitive(p) => write!(f, "{p}"),
            MonomorphicType::Var(v) => write!(f, "{v}"),
            MonomorphicType::Fn(input, output) => match input.as_ref() {
                MonomorphicType::Fn(_, _) => write!(f, "({input}) -> {output}"),
                _ => write!(f, "{input} -> {output}"),
            },
            MonomorphicType::Tuple(Tuple(vec)) => write!(f, "({})", vec.iter().join(", ")),
            MonomorphicType::Pointer(t) => write!(f, "*{t}"),
            MonomorphicType::Constant(id) => write!(f, "{id}"),
        }
    }
}

impl Display for PolymorphicType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.bindings.is_empty() {
            return write!(f, "{}", self.binding_type);
        }
        write!(
            f,
            "∀{} => {}",
            self.bindings.iter().join(", "),
            self.binding_type
        )
    }
}

impl Debug for TVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for MonomorphicType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for PolymorphicType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(test)]
pub mod tests {
    use crate::r#type::{fun1, MonomorphicType, PolymorphicType, PrimitiveType, TVar, Tuple};
    use proptest::arbitrary::StrategyFor;
    use proptest::prelude::{any, prop, Arbitrary, BoxedStrategy, Just, Strategy};
    use proptest::strategy::{Map, Recursive};
    use proptest::{prop_oneof, proptest};
    use std::num::NonZeroU8;

    impl Arbitrary for PrimitiveType {
        type Parameters = ();

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            let leaf = prop_oneof![
                Just(PrimitiveType::Boolean),
                Just(PrimitiveType::F24),
                (1..=24u8).prop_map(|it| PrimitiveType::I(NonZeroU8::new(it).unwrap())),
                (1..=24u8).prop_map(|it| PrimitiveType::U(NonZeroU8::new(it).unwrap())),
                any::<u64>().prop_map(PrimitiveType::StaticString)
            ];

            leaf.prop_recursive(2, 3, 1, |inner| {
                (inner, any::<u64>())
                    .prop_map(|it| PrimitiveType::StaticArray(it.1, Box::new(it.0)))
                    .boxed()
            })
        }

        type Strategy = Recursive<
            PrimitiveType,
            fn(BoxedStrategy<PrimitiveType>) -> BoxedStrategy<PrimitiveType>,
        >;
    }

    impl Arbitrary for MonomorphicType {
        type Parameters = ();

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            let leaf = prop_oneof![
                any::<PrimitiveType>().prop_map(MonomorphicType::Primitive),
                any::<TVar>().prop_map(MonomorphicType::Var),
                "_{0,2}[A-Z]([A-Za-z0-9_]){0,5}".prop_map(MonomorphicType::Constant)
            ];

            leaf.prop_recursive(10, 100, 10, |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..5)
                        .prop_map(|it| MonomorphicType::Tuple(Tuple(it))),
                    inner
                        .clone()
                        .prop_map(|it| MonomorphicType::Pointer(Box::new(it))),
                    (inner.clone(), inner).prop_map(|it| fun1(it.0, it.1))
                ]
                .boxed()
            })
        }

        type Strategy = Recursive<
            MonomorphicType,
            fn(BoxedStrategy<MonomorphicType>) -> BoxedStrategy<MonomorphicType>,
        >;
    }

    impl Arbitrary for PolymorphicType {
        type Parameters = ();

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            any::<MonomorphicType>().prop_map(|it| it.normalize())
        }

        type Strategy = Map<StrategyFor<MonomorphicType>, fn(MonomorphicType) -> PolymorphicType>;
    }

    proptest! {
        #[test]
        fn test(primitive in any::<PolymorphicType>()) {
            dbg!(primitive);
        }
    }
}

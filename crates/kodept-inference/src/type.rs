use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::iter::once;
use std::ops::{BitAnd, BitOr, Sub};

use derive_more::{Constructor, Display as DeriveDisplay, From};
use itertools::Itertools;
use nonempty_collections::NEVec;

use crate::{Environment, LOWER_ALPHABET};
use crate::substitution::Substitutions;

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

#[derive(Debug, Clone, PartialEq, DeriveDisplay, Eq, Hash)]
pub enum PrimitiveType {
    Integral,
    Floating,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq, From)]
pub struct TVar(pub(crate) usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Constructor)]

pub struct Tuple(pub(crate) Vec<MonomorphicType>);

#[derive(Debug, Clone, PartialEq, From, Eq, Hash)]
pub enum MonomorphicType {
    Primitive(PrimitiveType),
    Var(TVar),
    #[from(ignore)]
    Fn {
        input: Box<MonomorphicType>,
        output: Box<MonomorphicType>,
    },
    Tuple(Tuple),
    Pointer(Box<MonomorphicType>),
    Constant(String),
}

#[derive(Debug, Clone, PartialEq, From)]
#[from(forward)]
pub enum PolymorphicType {
    Monomorphic(MonomorphicType),
    #[from(ignore)]
    Binding {
        bind: TVar,
        binding_type: Box<PolymorphicType>,
    },
}

pub fn fun1<M: Into<MonomorphicType>, N: Into<MonomorphicType>>(
    input: N,
    output: M,
) -> MonomorphicType {
    MonomorphicType::Fn {
        input: Box::new(input.into()),
        output: Box::new(output.into()),
    }
}

pub fn fun<M: Into<MonomorphicType>>(input: NEVec<MonomorphicType>, output: M) -> MonomorphicType {
    if input.tail.is_empty() {
        return fun1(input.head, output);
    }

    let (first, body, last) = input.split();
    once(first)
        .chain(body)
        .cloned()
        .rfold(fun1(last.clone(), output), fun1)
}

pub fn var<V: Into<TVar>>(id: V) -> MonomorphicType {
    MonomorphicType::Var(id.into())
}

pub fn unit_type() -> MonomorphicType {
    MonomorphicType::Tuple(Tuple(vec![]))
}

impl MonomorphicType {
    #[must_use]
    pub fn free_types(&self) -> HashSet<TVar> {
        match self {
            MonomorphicType::Primitive(_) | MonomorphicType::Constant(_) => HashSet::new(),
            MonomorphicType::Var(x) => HashSet::from([x.clone()]),
            MonomorphicType::Fn { input, output } => input.free_types().bitor(&output.free_types()),
            MonomorphicType::Tuple(Tuple(vec)) => vec
                .iter()
                .fold(HashSet::new(), |acc, next| acc.bitor(&next.free_types())),
            MonomorphicType::Pointer(t) => t.free_types(),
        }
    }

    fn rename(self, old: usize, new: usize) -> Self {
        match self {
            MonomorphicType::Var(TVar(id)) if id == old => TVar(new).into(),
            MonomorphicType::Primitive(_)
            | MonomorphicType::Var(_)
            | MonomorphicType::Constant(_) => self,
            MonomorphicType::Fn { input, output } => MonomorphicType::Fn {
                input: Box::new(input.rename(old, new)),
                output: Box::new(output.rename(old, new)),
            },
            MonomorphicType::Tuple(Tuple(vec)) => MonomorphicType::Tuple(Tuple(
                vec.into_iter().map(|it| it.rename(old, new)).collect(),
            )),
            MonomorphicType::Pointer(t) => MonomorphicType::Pointer(Box::new(t.rename(old, new))),
        }
    }

    #[must_use]
    pub fn substitute(&self, from: &Substitutions) -> MonomorphicType {
        match self {
            MonomorphicType::Primitive(_) | MonomorphicType::Constant(_) => self.clone(),
            MonomorphicType::Var(_) => from
                .0
                .iter()
                .find(|it| it.substituted.eq(self))
                .map(|it| it.replacement.clone())
                .unwrap_or(self.clone()),
            MonomorphicType::Fn { input, output } => MonomorphicType::Fn {
                input: Box::new(input.substitute(from)),
                output: Box::new(output.substitute(from)),
            },
            MonomorphicType::Tuple(Tuple(vec)) => {
                MonomorphicType::Tuple(Tuple(vec.iter().map(|it| it.substitute(from)).collect()))
            }
            MonomorphicType::Pointer(t) => MonomorphicType::Pointer(Box::new(t.substitute(from))),
        }
    }

    fn extract_vars(&self) -> Vec<usize> {
        match self {
            MonomorphicType::Primitive(_) | MonomorphicType::Constant(_) => vec![],
            MonomorphicType::Var(TVar(x)) => vec![*x],
            MonomorphicType::Fn { input, output } => {
                let mut vec = vec![];
                vec.append(&mut input.extract_vars());
                vec.append(&mut output.extract_vars());
                vec
            }
            MonomorphicType::Tuple(Tuple(vec)) => {
                vec.iter().flat_map(MonomorphicType::extract_vars).collect()
            }
            MonomorphicType::Pointer(t) => t.extract_vars(),
        }
    }

    #[must_use]
    pub fn minimize(self) -> MonomorphicType {
        let mut vars = self.extract_vars();
        vars.sort_unstable();
        vars.dedup();

        if let Some(0) = vars.first() {
            return self;
        }

        vars.into_iter()
            .zip(0..)
            .fold(self, |acc, (old, new)| acc.rename(old, new))
    }
}

impl PolymorphicType {
    fn collect(&self) -> (Vec<TVar>, MonomorphicType) {
        let mut result = vec![];
        let mut current = self;
        loop {
            match current {
                PolymorphicType::Monomorphic(ty) => return (result, ty.clone()),
                PolymorphicType::Binding { bind, binding_type } => {
                    result.push(bind.clone());
                    current = binding_type.as_ref();
                }
            }
        }
    }

    #[must_use]
    pub fn free_types(&self) -> HashSet<TVar> {
        match self {
            PolymorphicType::Monomorphic(t) => t.free_types(),
            PolymorphicType::Binding { bind, binding_type } => binding_type
                .free_types()
                .sub(&HashSet::from([bind.clone()])),
        }
    }

    #[must_use]
    pub fn substitute(&self, from: &Substitutions) -> PolymorphicType {
        match self {
            PolymorphicType::Monomorphic(t) => PolymorphicType::Monomorphic(t.substitute(from)),
            PolymorphicType::Binding { bind, binding_type } => PolymorphicType::Binding {
                bind: bind.clone(),
                binding_type: Box::new(
                    binding_type.substitute(&Substitutions(
                        from.0
                            .iter()
                            .filter(|it| it.replacement != MonomorphicType::Var(bind.clone()))
                            .cloned()
                            .collect(),
                    )),
                ),
            },
        }
    }

    pub fn instantiate(&self, env: &mut Environment) -> MonomorphicType {
        match self {
            PolymorphicType::Monomorphic(t) => t.clone(),
            PolymorphicType::Binding { bind, binding_type } => match binding_type.as_ref() {
                PolymorphicType::Monomorphic(t) => t.substitute(&Substitutions::single(
                    MonomorphicType::Var(bind.clone()),
                    MonomorphicType::Var(env.new_var()),
                )),
                PolymorphicType::Binding { .. } => binding_type.instantiate(env),
            },
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
        self.substitute(&rhs)
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
        self.substitute(&rhs)
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
        write!(f, "'{}", expand_to_string(self.0, LOWER_ALPHABET))
    }
}

impl Display for MonomorphicType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MonomorphicType::Primitive(p) => write!(f, "{p}"),
            MonomorphicType::Var(v) => write!(f, "{v}"),
            MonomorphicType::Fn { input, output } => match input.as_ref() {
                MonomorphicType::Fn { .. } => write!(f, "({input}) -> {output}"),
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
        match self {
            PolymorphicType::Monomorphic(t) => write!(f, "{t}"),
            PolymorphicType::Binding { .. } => {
                let (bindings, t) = self.collect();
                let pretty_bindings = || bindings.into_iter().rev().map(|it| it.0).zip(0usize..);
                let renamed =
                    pretty_bindings.clone()().fold(t, |acc, (old, new)| acc.rename(old, new));
                write!(
                    f,
                    "∀{} => {}",
                    pretty_bindings()
                        .map(|it| format!("'{}", expand_to_string(it.1, LOWER_ALPHABET)))
                        .join(", "),
                    renamed
                )
            }
        }
    }
}

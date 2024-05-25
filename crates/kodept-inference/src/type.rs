use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::fmt::{Display, Formatter};
use std::iter::once;
use std::ops::BitAnd;

use derive_more::{Constructor, Display as DeriveDisplay, From};
use itertools::{concat, Itertools};
use nonempty_collections::NEVec;

use PolymorphicType::Monomorphic;

use crate::r#type::PolymorphicType::Binding;
use crate::substitution::Substitutions;
use crate::traits::{FreeTypeVars, Substitutable};
use crate::{Environment, LOWER_ALPHABET};

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

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, From)]
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

impl MonomorphicType {
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

    fn extract_vars(&self) -> Vec<usize> {
        match self {
            MonomorphicType::Primitive(_) | MonomorphicType::Constant(_) => vec![],
            MonomorphicType::Var(TVar(x)) => vec![*x],
            MonomorphicType::Fn { input, output } => {
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
        diff.into_iter()
            .rfold(self.clone().into(), |acc, next| Binding {
                bind: next,
                binding_type: Box::new(acc),
            })
    }

    pub fn normalize(self) -> PolymorphicType {
        self.generalize(&HashSet::new()).normalize()
    }
}

impl PolymorphicType {
    fn collect(&self) -> (Vec<TVar>, MonomorphicType) {
        let mut result = vec![];
        let mut current = self;
        loop {
            match current {
                Monomorphic(ty) => return (result, ty.clone()),
                Binding { bind, binding_type } => {
                    result.push(bind.clone());
                    current = binding_type.as_ref();
                }
            }
        }
    }

    pub fn normalize(self) -> Self {
        match self {
            Monomorphic(_) => self,
            Binding { .. } => {
                let (_, body) = self.collect();
                let mut free = body.extract_vars();
                free.sort_unstable();
                free.dedup();
                let len = free.len();
                let body = free
                    .iter()
                    .zip(0usize..)
                    .fold(body, |acc, (&old, new)| acc.rename(old, new));
                free.into_iter()
                    .zip(0usize..len)
                    .map(|it| TVar(it.1))
                    .rfold(body.into(), |acc, next| Binding {
                        bind: next,
                        binding_type: Box::new(acc),
                    })
            }
        }
    }

    pub fn instantiate(&self, env: &mut Environment) -> MonomorphicType {
        match self {
            Monomorphic(t) => t.clone(),
            Binding { bind, binding_type } => match binding_type.as_ref() {
                Monomorphic(t) => t.substitute(&Substitutions::single(
                    env.new_var(),
                    MonomorphicType::Var(bind.clone()),
                )),
                Binding { .. } => binding_type.instantiate(env),
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
        write!(f, "{}", expand_to_string(self.0, LOWER_ALPHABET))
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
            Monomorphic(t) => write!(f, "{t}"),
            Binding { .. } => {
                let (bindings, t) = self.collect();
                write!(
                    f,
                    "∀{} => {}",
                    bindings
                        .into_iter()
                        .map(|it| format!("{}", expand_to_string(it.0, LOWER_ALPHABET)))
                        .join(", "),
                    t
                )
            }
        }
    }
}

use std::fmt::{Debug, Display, Formatter};

use crate::r#type::MonomorphicType;
use derive_more::{Display, From};
use itertools::Itertools;

#[derive(PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct BVar {
    pub var: Var,
    pub ty: Option<MonomorphicType>,
}

#[derive(Display, Clone, PartialEq, Eq, Hash)]
#[display("{name}")]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Var {
    #[cfg_attr(test, proptest(regex = "_{0,2}[a-z]([A-Za-z0-9_]){0,5}"))]
    pub name: String,
}

#[derive(PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[cfg_attr(test, proptest(no_params))]
pub struct App {
    pub arg: Box<Language>,
    pub func: Box<Language>,
}

#[derive(PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[cfg_attr(test, proptest(no_params))]
pub struct Lambda {
    pub bind: BVar,
    pub expr: Box<Language>,
}

#[derive(PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[cfg_attr(test, proptest(no_params))]
pub struct Let {
    pub binder: Box<Language>,
    pub bind: BVar,
    pub usage: Box<Language>,
}

#[derive(PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[cfg_attr(test, proptest(no_params))]
pub enum Literal {
    Integral,
    Floating,
    Tuple(Vec<Language>),
}

#[derive(PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[cfg_attr(test, proptest(no_params))]
pub enum Special {
    If {
        condition: Box<Language>,
        body: Box<Language>,
        otherwise: Box<Language>,
    },
}

#[derive(From, PartialEq, Eq, Hash)]
pub enum Language {
    Var(Var),
    App(App),
    Lambda(Lambda),
    Let(Let),
    Literal(Literal),
    Special(Special),
}

pub fn var<V: Into<Var>>(id: V) -> Var {
    id.into()
}

pub fn app<N: Into<Language>, M: Into<Language>>(arg: N, func: M) -> App {
    App {
        arg: Box::new(arg.into()),
        func: Box::new(func.into()),
    }
}

pub fn lambda<B, E>(bind: B, expr: E) -> Lambda
where
    BVar: From<B>,
    E: Into<Language>,
{
    Lambda {
        bind: bind.into(),
        expr: Box::new(expr.into()),
    }
}

pub fn r#let<V, B, U>(bind: V, binder: B, usage: U) -> Let
where
    B: Into<Language>,
    U: Into<Language>,
    BVar: From<V>,
{
    Let {
        binder: Box::new(binder.into()),
        bind: bind.into(),
        usage: Box::new(usage.into()),
    }
}

pub fn r#if(
    condition: impl Into<Language>,
    body: impl Into<Language>,
    otherwise: impl Into<Language>,
) -> Special {
    Special::If {
        condition: Box::new(condition.into()),
        body: Box::new(body.into()),
        otherwise: Box::new(otherwise.into()),
    }
}

pub fn bounded(v: impl Into<Var>, t: impl Into<MonomorphicType>) -> BVar {
    BVar {
        var: v.into(),
        ty: Some(t.into()),
    }
}

impl<S: Into<Var>> From<S> for BVar {
    fn from(value: S) -> Self {
        Self {
            var: value.into(),
            ty: None,
        }
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Var(x) => Display::fmt(x, f),
            Language::App(x) => Display::fmt(x, f),
            Language::Lambda(x) => Display::fmt(x, f),
            Language::Let(x) => Display::fmt(x, f),
            Language::Literal(x) => Display::fmt(x, f),
            Language::Special(x) => Display::fmt(x, f),
        }
    }
}

impl Display for App {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.arg.as_ref() {
            Language::Var(_) | Language::Literal(_) => match self.func.as_ref() {
                Language::Var(_) | Language::Literal(_) => write!(f, "{} {}", self.func, self.arg),
                Language::App(App { func, .. }) => match func.as_ref() {
                    Language::App(_) => write!(f, "{} ({})", self.func, self.arg),
                    _ => write!(f, "{} {}", self.func, self.arg),
                },
                _ => write!(f, "{} ({})", self.func, self.arg),
            },
            _ => write!(f, "{} ({})", self.func, self.arg),
        }
    }
}

impl Display for Lambda {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "λ{}. {}", self.bind, self.expr)
    }
}

impl Display for Let {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(let {} = {} in {})", self.bind, self.binder, self.usage)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Integral => write!(f, "<integral>"),
            Literal::Floating => write!(f, "<floating>"),
            Literal::Tuple(t) => write!(f, "({})", t.iter().join(", ")),
        }
    }
}

impl Display for Special {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Special::If {
                condition,
                body,
                otherwise,
            } => write!(f, "if ({condition}) ({body}) ({otherwise})"),
        }
    }
}

impl Display for BVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.ty {
            None => write!(f, "{}", self.var),
            Some(ty) => write!(f, "{} :: {}", self.var, ty),
        }
    }
}

impl Debug for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Var(x) => Debug::fmt(x, f),
            Language::App(x) => Debug::fmt(x, f),
            Language::Lambda(x) => Debug::fmt(x, f),
            Language::Let(x) => Debug::fmt(x, f),
            Language::Literal(x) => Debug::fmt(x, f),
            Language::Special(x) => Debug::fmt(x, f),
        }
    }
}

impl Debug for App {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for Lambda {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for Let {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for Special {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for BVar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl Debug for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl<S: Into<String>> From<S> for Var {
    fn from(value: S) -> Self {
        Self { name: value.into() }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::assumption::Environment;
    use crate::language::{app, lambda, r#let, var, BVar, Language, Literal, Special, Var};
    use crate::r#type::{fun1, tuple, var as t_var};
    use proptest::collection::vec;
    use proptest::prelude::{any, Arbitrary, BoxedStrategy, Strategy};
    use proptest::prop_oneof;
    use proptest::strategy::{LazyJust, Recursive};
    use std::collections::HashSet;

    impl Arbitrary for Language {
        type Parameters = ();

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            let leaf = prop_oneof![
                1 => LazyJust::new(|| Language::Literal(Literal::Floating)),
                1 => LazyJust::new(|| Language::Literal(Literal::Integral)),
                5 => any::<Var>().prop_map(Language::Var),
            ];

            leaf.prop_recursive(100, 100, 100, |inner| {
                prop_oneof![
                    2 => vec(inner.clone(), 0..=5).prop_map(|it| Language::Literal(Literal::Tuple(it))),
                    2 => (inner.clone(), inner.clone()).prop_map(|it| app(it.0, it.1).into()),
                    1 => (inner.clone(), inner.clone(), inner.clone()).prop_map(|it| Language::Special(Special::If {
                        condition: Box::new(it.0),
                        body: Box::new(it.1),
                        otherwise: Box::new(it.2)
                    })),
                    2 => (inner.clone(), any::<BVar>(), inner.clone()).prop_map(|it| r#let(it.1, it.0, it.2).into()),
                    2 => (any::<BVar>(), inner.clone()).prop_map(|it| lambda(it.0, it.1).into())
                ].boxed()
            })
        }

        type Strategy = Recursive<Language, fn(BoxedStrategy<Language>) -> BoxedStrategy<Language>>;
    }

    #[test]
    fn test_infer_language() {
        // λz. let x = (z, z) in (λy. (y, y)) x
        // ∀a, b, c => a -> ((a, a), (a, a))
        let expr: Language = lambda(
            "z",
            r#let(
                "x",
                Literal::Tuple(vec![var("z").into(), var("z").into()]),
                app(
                    var("x"),
                    lambda("y", Literal::Tuple(vec![var("y").into(), var("y").into()])),
                ),
            ),
        )
        .into();

        let t = expr.infer(&Environment::empty()).unwrap();

        println!("{}\n{}", expr, t);
        assert_eq!(
            t,
            fun1(
                t_var(0),
                tuple([tuple([t_var(0), t_var(0)]), tuple([t_var(0), t_var(0)])])
            )

            .generalize(&HashSet::new())
        );
    }

    #[test]
    fn test_church_encoding() {
        //zero = \f. \x. x                   :: a -> b -> b
        //one  = \f. \x. f x                 :: (a -> b) -> a -> b
        //plus = \m. \n. \f. \x. m f (n f x) :: (a -> b -> c) -> (a -> d -> b) -> a -> d -> c

        let zero: Language = lambda("f", lambda("x", var("x"))).into();
        let one: Language = lambda("f", lambda("x", app(var("x"), var("f")))).into();
        let plus: Language = lambda(
            "m",
            lambda(
                "n",
                lambda(
                    "f",
                    lambda(
                        "x",
                        app(
                            app(var("x"), app(var("f"), var("n"))),
                            app(var("f"), var("m")),
                        ),
                    ),
                ),
            ),
        )
        .into();

        let zt = zero.infer(&Environment::empty()).unwrap();
        let ot = one.infer(&Environment::empty()).unwrap();
        let pt = plus.infer(&Environment::empty()).unwrap();

        println!("{}\n{}\n\n{}\n{}\n\n{}\n{}", zero, zt, one, ot, plus, pt);
    }
}

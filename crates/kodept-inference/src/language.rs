use derive_more::{Display, From};
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use itertools::Itertools;

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
#[display(fmt = "{name}")]
pub struct Var {
    name: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct App {
    pub arg: Box<Language>,
    pub func: Box<Language>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Lambda {
    pub bind: Rc<Language>, // only vars,
    pub expr: Box<Language>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Let {
    pub binder: Box<Language>,
    pub bind: Rc<Language>, // only vars,
    pub usage: Box<Language>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Literal {
    Integral(String),
    Floating(String),
    Tuple(Vec<Language>),
    Union(Vec<Language>),
}

#[derive(Debug, From, Display, PartialEq, Eq, Hash)]
pub enum Language {
    Var(Var),
    App(App),
    Lambda(Lambda),
    Let(Let),
    Literal(Literal),
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
    Var: From<B>,
    E: Into<Language>,
{
    Lambda {
        bind: Rc::new(Var::from(bind).into()),
        expr: Box::new(expr.into()),
    }
}

pub fn r#let<V, B, U>(bind: V, binder: B, usage: U) -> Let
where
    B: Into<Language>,
    U: Into<Language>,
    Var: From<V>,
{
    Let {
        binder: Box::new(binder.into()),
        bind: Rc::new(Var::from(bind).into()),
        usage: Box::new(usage.into()),
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
        write!(f, "let {} = {} in {}", self.bind, self.binder, self.usage)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Integral(n) | Literal::Floating(n) => write!(f, "{n}"),
            Literal::Tuple(t) => write!(f, "({})", t.iter().join(", ")),
            Literal::Union(u) => write!(f, "({})", u.iter().join(" | ")),
        }
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
    use crate::language::{app, lambda, r#let, var, Language, Literal};
    use crate::r#type::{fun1, var as t_var, Tuple, Union};

    #[test]
    fn test_infer_language() {
        //λz. let x = (z | z) in (λy. (y, y)) x
        let expr: Language = lambda(
            "z",
            r#let(
                "x",
                Literal::Union(vec![var("z").into(), var("z").into()]),
                app(
                    var("x"),
                    lambda("y", Literal::Tuple(vec![var("y").into(), var("y").into()])),
                ),
            ),
        )
        .into();

        let t = expr.infer_type().unwrap();

        assert_eq!(
            t,
            fun1(
                t_var(0),
                Tuple(vec![
                    Union(vec![t_var(0), t_var(0)]).into(),
                    Union(vec![t_var(0), t_var(0)]).into()
                ])
            )
        );

        println!("{}\n{}", expr, t.minimize());
    }

    #[test]
    fn test_church_encoding() {
        //zero = \f. \x. x
        //one  = \f. \x. f x
        //plus = \m. \n. \f. \x. m f (n f x)

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

        let zt = zero.infer_type().unwrap();
        let ot = one.infer_type().unwrap();
        let pt = plus.infer_type().unwrap();

        println!(
            "{}\n{}\n\n{}\n{}\n\n{}\n{}",
            zero,
            zt.minimize(),
            one,
            ot.minimize(),
            plus,
            pt.minimize()
        );
    }
}

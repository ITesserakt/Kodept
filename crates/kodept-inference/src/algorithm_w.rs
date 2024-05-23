use std::rc::Rc;

use derive_more::From;

use crate::{Environment, language};
use crate::algorithm_u::AlgorithmUError;
use crate::assumption::Assumptions;
use crate::language::{Language, Literal, Special};
use crate::r#type::{fun1, MonomorphicType, PrimitiveType, Tuple, var};
use crate::substitution::Substitutions;

#[derive(From, Debug)]
pub enum AlgorithmWError {
    AlgorithmU(AlgorithmUError),
    #[from(ignore)]
    UnknownVar(language::Var),
}

struct AlgorithmW<'e> {
    context: &'e mut Assumptions,
    env: &'e mut Environment,
}

type AWResult = Result<(Substitutions, MonomorphicType), AlgorithmWError>;

impl<'e> AlgorithmW<'e> {
    fn apply(&mut self, expression: &Language) -> AWResult {
        match expression {
            Language::Var(x) => self.apply_var(x),
            Language::App(x) => self.apply_app(x),
            Language::Lambda(x) => self.apply_lambda(x),
            Language::Let(x) => self.apply_let(x),
            Language::Special(x) => self.apply_special(x),
            Language::Literal(x) => match x {
                Literal::Integral(_) => {
                    Ok((Substitutions::empty(), PrimitiveType::Integral.into()))
                }
                Literal::Floating(_) => {
                    Ok((Substitutions::empty(), PrimitiveType::Floating.into()))
                }
                Literal::Tuple(vec) => self.apply_tuple(vec),
            },
        }
    }

    fn apply_var(&mut self, var: &language::Var) -> AWResult {
        match self.context.get(&Language::Var(var.clone())) {
            None => Err(AlgorithmWError::UnknownVar(var.clone())),
            Some(x) => Ok((Substitutions::empty(), x.instantiate(self.env))),
        }
    }

    fn apply_app(&mut self, language::App { arg, func }: &language::App) -> AWResult {
        let v = self.env.new_var();
        let (s1, t1) = func.infer_w(self)?;
        let (s2, t2) = arg.infer_w(self)?;
        let s3 = (t1 & &s2).unify(&fun1(t2, var(v.clone())))?;
        Ok((&s3 + s2 + s1, var(v) & s3))
    }

    fn apply_lambda(&mut self, language::Lambda { bind, expr }: &language::Lambda) -> AWResult {
        let v = self.env.new_var();
        let var_bind = bind.clone();
        let mut new_context = self.context.clone();
        new_context.push(var_bind.clone(), Rc::new(MonomorphicType::Var(v.clone()).into()));
        let (s1, t1) = AlgorithmW {
            context: &mut new_context,
            env: self.env,
        }.apply(expr)?;
        let resulting_fn = fun1(var(v) & &s1, t1 & &s1);
        Ok((s1, resulting_fn))
    }

    fn apply_let(
        &mut self,
        language::Let {
            binder,
            bind,
            usage,
        }: &language::Let,
    ) -> AWResult {
        let (s1, t1) = binder.infer_w(self)?;
        let var_bind = bind.clone();
        let poly_type = self.context.substitute_mut(&s1).generalize(t1);
        let mut new_context = self.context.clone();
        new_context
            .retain_all(match bind.as_ref() {
                Language::Var(v) => v,
                _ => unreachable!(),
            })
            .push(var_bind, Rc::new(poly_type));

        let (s2, t2) = AlgorithmW {
            context: &mut new_context,
            env: self.env,
        }
        .apply(usage.as_ref())?;
        Ok((s2 + s1, t2))
    }

    fn apply_tuple(&mut self, tuple: &[Language]) -> AWResult {
        tuple
            .iter()
            .try_fold(
                (Substitutions::empty(), Tuple::unit()),
                |(s, mut t), next| {
                    self.context.substitute_mut(&s);
                    let (s1, t1) = next.infer_w(self)?;
                    t.push(t1);
                    Ok((s + s1, t))
                },
            )
            .map(|(s, t)| (s, t.into()))
    }

    fn apply_special(&mut self, special: &Special) -> AWResult {
        match special {
            Special::If {
                condition,
                body,
                otherwise,
            } => {
                let (s1, t_cond) = condition.infer_w(self)?;
                let (s2, t_true) = body.infer_w(self)?;
                let (s3, t_false) = otherwise.infer_w(self)?;
                let s4 = (t_cond & &s1).unify(&PrimitiveType::Boolean.into())?;
                let s5 = (&t_true & &s3).unify(&(t_false & &s3))?;
                Ok((s5 + &s4 + s3 + s2 + s1, t_true & s4))
            }
        }
    }
}

impl Language {
    fn infer_w(
        &self,
        context: &mut AlgorithmW,
    ) -> Result<(Substitutions, MonomorphicType), AlgorithmWError> {
        context.apply(self)
    }

    pub fn infer_with_env(
        self: Rc<Self>,
        context: &mut Assumptions,
        env: &mut Environment,
    ) -> Result<MonomorphicType, AlgorithmWError> {
        let (s, t) = AlgorithmW { context, env }.apply(&self)?;
        let t = t.substitute(&s);
        let poly_type = context.generalize(t.clone());
        context.substitute_mut(&s).push(self, Rc::new(poly_type));
        Ok(t)
    }

    pub fn infer(self: Rc<Self>) -> Result<(Assumptions, MonomorphicType), AlgorithmWError> {
        let mut assumptions = Assumptions::empty();
        let t = Self::infer_with_env(
            self,
            &mut assumptions,
            &mut Environment { variable_index: 0 },
        )?;
        Ok((assumptions, t))
    }

    pub fn infer_type(&self) -> Result<MonomorphicType, AlgorithmWError> {
        let mut context = Assumptions::empty();
        let mut env = Environment::default();
        AlgorithmW {
            context: &mut context,
            env: &mut env,
        }
        .apply(self)
        .map(|it| it.1)
    }
}

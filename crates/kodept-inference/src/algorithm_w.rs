use std::rc::Rc;

use derive_more::From;

use crate::algorithm_u::AlgorithmUError;
use crate::assumption::Assumptions;
use crate::language::{Language, Literal, Special};
use crate::r#type::{fun1, var, MonomorphicType, PrimitiveType, Tuple};
use crate::substitution::Substitutions;
use crate::{language, Environment};

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

    fn apply_app(&mut self, app: &language::App) -> AWResult {
        let v = self.env.new_var();
        let (s1, t1) = app.func.infer_w(self)?;
        let (s2, t2) = app.arg.infer_w(self)?;
        let s3 = t1.substitute(&s1).unify(&fun1(t2, var(v.clone())))?;
        Ok((s3.compose(&s2).compose(&s1), var(v).substitute(&s3)))
    }

    fn apply_lambda(&mut self, lambda: &language::Lambda) -> AWResult {
        let v = self.env.new_var();
        let var_bind = lambda.bind.clone();
        let mut new_context = self.context.clone();
        new_context.push(var_bind, Rc::new(MonomorphicType::Var(v.clone()).into()));
        let (s1, t1) = AlgorithmW {
            context: &mut new_context,
            env: self.env,
        }
        .apply(&lambda.expr)?;
        let resulting_fn = fun1(var(v).substitute(&s1), t1);
        Ok((s1, resulting_fn))
    }

    fn apply_let(&mut self, l: &language::Let) -> AWResult {
        let (s1, t1) = l.binder.infer_w(self)?;
        let var_bind = l.bind.clone();
        let poly_type = self.context.substitute_mut(&s1).generalize(t1);
        let mut new_context = self.context.clone();
        new_context
            .filter_all(match l.bind.as_ref() {
                Language::Var(v) => v,
                _ => unreachable!(),
            })
            .push(var_bind, Rc::new(poly_type));

        let (s2, t2) = AlgorithmW {
            context: &mut new_context,
            env: self.env,
        }
        .apply(l.usage.as_ref())?;
        Ok((s2.compose(&s1), t2))
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
                    Ok((s.compose(&s1), t))
                },
            )
            .map(|(s, t)| (s, t.into()))
    }

    fn apply_special(&mut self, special: &Special) -> AWResult {
        match special {
            Special::If { condition, body, otherwise } => {
                let (s1, t_cond) = condition.infer_w(self)?;
                let s2 = t_cond.unify(&PrimitiveType::Boolean.into())?;
                let (s3, t1) = body.infer_w(self)?;
                let (s4, t2) = otherwise.infer_w(self)?;
                let s5 = t1.unify(&t2)?;
                let s6 = s4.compose(&s5);
                Ok((s1.compose(&s2).compose(&s3).compose(&s6), t1.substitute(&s6)))
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

use derive_more::From;

use crate::{Environment, language};
use crate::algorithm_u::AlgorithmUError;
use crate::assumption::Assumptions;
use crate::language::{Language, Literal};
use crate::r#type::{fun1, MonomorphicType, PrimitiveType, Tuple, Union, var};
use crate::substitution::Substitutions;

#[derive(From, Debug)]
pub enum AlgorithmWError {
    AlgorithmU(AlgorithmUError),
    #[from(ignore)]
    UnknownVar(language::Var),
}

struct AlgorithmW<'e, 'l> {
    context: &'e mut Assumptions<'l>,
    env: &'e mut Environment,
}

type AWResult = Result<(Substitutions, MonomorphicType), AlgorithmWError>;

impl<'e, 'l> AlgorithmW<'e, 'l> {
    fn apply(&mut self, expression: &Language) -> AWResult {
        match expression {
            Language::Var(x) => self.apply_var(x),
            Language::App(x) => self.apply_app(x),
            Language::Lambda(x) => self.apply_lambda(x),
            Language::Let(x) => self.apply_let(x),
            Language::Literal(x) => match x {
                Literal::Integral(_) => {
                    Ok((Substitutions::empty(), PrimitiveType::Integral.into()))
                }
                Literal::Floating(_) => {
                    Ok((Substitutions::empty(), PrimitiveType::Floating.into()))
                }
                Literal::Tuple(vec) => self.apply_tuple(vec),
                Literal::Union(vec) => self.apply_union(vec),
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
        let var_bind = lambda.bind.clone().into();
        let mut new_context = self.context.clone();
        new_context.push(&var_bind, MonomorphicType::Var(v.clone()).into());
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
        let var_bind = l.bind.clone().into();
        let poly_type = self.context.substitute_mut(&s1).generalize(t1);
        let mut new_context = self.context.clone();
        new_context.filter_all(&l.bind).push(&var_bind, poly_type);

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

    fn apply_union(&mut self, tuple: &[Language]) -> AWResult {
        tuple
            .iter()
            .try_fold(
                (Substitutions::empty(), Union::bottom()),
                |(s, mut t), next| {
                    self.context.substitute_mut(&s);
                    let (s1, t1) = next.infer_w(self)?;
                    t.push(t1);
                    Ok((s.compose(&s1), t))
                },
            )
            .map(|(s, t)| (s, t.into()))
    }
}

impl Language {
    fn infer_w(
        &self,
        context: &mut AlgorithmW,
    ) -> Result<(Substitutions, MonomorphicType), AlgorithmWError> {
        context.apply(self)
    }

    pub fn infer_with_env<'l>(
        &'l self,
        context: &mut Assumptions<'l>,
        env: &mut Environment,
    ) -> Result<MonomorphicType, AlgorithmWError> {
        let (s, t) = AlgorithmW { context, env }.apply(self)?;
        let poly_type = context.generalize(t.clone());
        context.substitute_mut(&s).push(self, poly_type);
        Ok(t)
    }

    pub fn infer(&self) -> Result<(Assumptions, MonomorphicType), AlgorithmWError> {
        let mut assumptions = Assumptions::empty();
        let t = self.infer_with_env(&mut assumptions, &mut Environment { variable_index: 0 })?;
        Ok((assumptions, t))
    }
}

use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use itertools::{concat, Itertools};
use nonempty_collections::{IteratorExt, NEVec, NonEmptyIterator};
use thiserror::Error;
use tracing::debug;

use crate::algorithm_u::AlgorithmUError;
use crate::algorithm_w::AlgorithmWError::UnknownVar;
use crate::assumption::{Assumptions, Assumptions2};
use crate::constraint::{eq_cst, explicit_cst, implicit_cst, Constraint, ConstraintsSolverError};
use crate::language::{Language, Literal, Special, Var};
use crate::r#type::PrimitiveType::Boolean;
use crate::r#type::{fun1, MonomorphicType, PolymorphicType, PrimitiveType, TVar, Tuple};
use crate::substitution::Substitutions;
use crate::traits::Substitutable;
use crate::{language, Environment};

#[derive(Debug, Error)]
pub enum AlgorithmWError {
    #[error(transparent)]
    AlgorithmU(#[from] AlgorithmUError),
    UnknownVar(NEVec<Var>),
    #[error(transparent)]
    FailedConstraints(#[from] ConstraintsSolverError),
}

struct AlgorithmW<'e> {
    context: HashSet<TVar>,
    env: &'e mut Environment,
}

type AWResult = Result<(Assumptions2, Vec<Constraint>, MonomorphicType), AlgorithmWError>;

impl<'e> AlgorithmW<'e> {
    fn apply(&mut self, expression: &Language) -> AWResult {
        match expression {
            Language::Var(x) => self.apply_var(x),
            Language::App(x) => self.apply_app(x),
            Language::Lambda(x) => self.apply_lambda(x),
            Language::Let(x) => self.apply_let(x),
            Language::Special(x) => self.apply_special(x),
            Language::Literal(x) => match x {
                Literal::Integral(_) => Ok((
                    Assumptions2::empty(),
                    vec![],
                    PrimitiveType::Integral.into(),
                )),
                Literal::Floating(_) => Ok((
                    Assumptions2::empty(),
                    vec![],
                    PrimitiveType::Floating.into(),
                )),
                Literal::Tuple(vec) => self.apply_tuple(vec),
            },
        }
    }

    fn apply_var(&mut self, var: &Var) -> AWResult {
        let fresh = self.env.new_var();
        Ok((Assumptions2::single(&var.name, fresh), vec![], fresh.into()))
    }

    fn apply_app(&mut self, language::App { arg, func }: &language::App) -> AWResult {
        let tv = self.env.new_var();
        let (as1, cs1, t1) = self.apply(func)?;
        let (as2, cs2, t2) = self.apply(arg)?;

        Ok((
            as1 + as2,
            concat([cs1, cs2, vec![eq_cst(t1, fun1(t2, tv))]]),
            tv.into(),
        ))
    }

    fn apply_lambda(&mut self, language::Lambda { bind, expr }: &language::Lambda) -> AWResult {
        let tv = self.env.new_var();
        let (as1, cs1, t1) = self.apply(expr)?;
        self.context.insert(tv);

        let mut as_ = as1.clone();
        as_.remove(bind);
        let eq_cs = as1
            .get(bind)
            .into_iter()
            .map(|it| eq_cst(it.clone(), tv))
            .collect();

        Ok((as_, concat([cs1, eq_cs]), fun1(tv, t1)))
    }

    fn apply_let(
        &mut self,
        language::Let {
            binder,
            bind,
            usage,
        }: &language::Let,
    ) -> AWResult {
        let (as1, cs1, t1) = self.apply(binder)?;
        let (as2, cs2, t2) = self.apply(usage)?;

        let mut as_ = as1 + &as2;
        as_.remove(bind);
        let im_cs = as2
            .get(bind)
            .into_iter()
            .map(|it| implicit_cst(it.clone(), self.context.clone(), t1.clone()))
            .collect();

        Ok((as_, concat([cs1, cs2, im_cs]), t2))
    }

    fn apply_tuple(&mut self, tuple: &[Language]) -> AWResult {
        let ctx: Vec<_> = tuple.into_iter().map(|it| self.apply(it)).try_collect()?;
        let (a, c, t): (Vec<_>, Vec<_>, Vec<_>) = ctx.into_iter().multiunzip();
        Ok((
            Assumptions2::merge_many(a.into_iter()),
            c.into_iter().flatten().collect(),
            Tuple(t).into(),
        ))
    }

    fn apply_special(&mut self, special: &Special) -> AWResult {
        match special {
            Special::If {
                condition,
                body,
                otherwise,
            } => {
                let (as1, cs1, t1) = self.apply(condition)?;
                let (as2, cs2, t2) = self.apply(body)?;
                let (as3, cs3, t3) = self.apply(otherwise)?;

                Ok((
                    as1 + as2 + as3,
                    concat([
                        cs1,
                        cs2,
                        cs3,
                        vec![eq_cst(t1, Boolean), eq_cst(t2.clone(), t3)],
                    ]),
                    t2,
                ))
            }
        }
    }
}

impl Language {
    fn infer_w(
        &self,
        context: &mut AlgorithmW,
        table: &Assumptions,
    ) -> Result<(Substitutions, MonomorphicType), AlgorithmWError> {
        let (a, c, t) = context.apply(self)?;
        if let Some(iter) = a
            .keys()
            .collect::<HashSet<_>>()
            .difference(&table.keys().collect())
            .to_nonempty_iter()
        {
            return Err(UnknownVar(iter.cloned().cloned().collect()));
        }
        let explicits: Vec<_> = table
            .iter()
            .flat_map(|(k, s)| {
                a.get(k)
                    .into_iter()
                    .map(|t| explicit_cst(t.clone(), s.clone()))
                    .collect::<Vec<_>>()
            })
            .collect();
        debug!("Inferred raw type and constraints: ");
        debug!(
            "[{}] ++ [{}], {t}",
            c.iter().join(", "),
            explicits.iter().join(", ")
        );
        let substitutions = Constraint::solve(&concat([c, explicits]), context.env)?;
        let t = t.substitute(&substitutions);
        debug!("Inferred type and substitutions: ");
        debug!("{}, {}", substitutions, t);
        Ok((substitutions, t))
    }

    pub fn infer_with_env(
        &self,
        context: &Assumptions,
        env: &mut Environment,
    ) -> Result<PolymorphicType, AlgorithmWError> {
        let mut ctx = AlgorithmW {
            context: Default::default(),
            env,
        };
        match self.infer_w(&mut ctx, context) {
            Ok((s, t)) => Ok(t.substitute(&s).normalize()),
            Err(e) => Err(e),
        }
    }

    pub fn infer(&self, table: &Assumptions) -> Result<PolymorphicType, AlgorithmWError> {
        self.infer_with_env(table, &mut Environment::default())
    }
}

impl Display for AlgorithmWError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgorithmWError::AlgorithmU(x) => write!(f, "{x}"),
            UnknownVar(vs) => write!(f, "Unknown references: [{}]", vs.iter().into_iter().join(", ")),
            AlgorithmWError::FailedConstraints(x) => write!(f, "{x}"),
        }
    }
}

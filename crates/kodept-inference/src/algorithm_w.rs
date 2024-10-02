use itertools::{concat, Itertools};
use nonempty_collections::NEVec;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use tracing::debug;

use crate::algorithm_u::AlgorithmUError;
use crate::algorithm_w::AlgorithmWError::UnknownVar;
use crate::assumption::AssumptionSet;
use crate::constraint::{eq_cst, explicit_cst, implicit_cst, Constraint, ConstraintsSolverError};
use crate::language::{Language, Literal, Special, Var};
use crate::r#type::PrimitiveType::Boolean;
use crate::r#type::{fun1, MonomorphicType, PolymorphicType, PrimitiveType, TVar, Tuple};
use crate::substitution::Substitutions;
use crate::traits::{EnvironmentProvider, Substitutable};
use crate::{language, InferState};

#[derive(Debug, Error)]
pub enum AlgorithmWError {
    #[error(transparent)]
    AlgorithmU(#[from] AlgorithmUError),
    UnknownVar(NEVec<Var>),
    #[error(transparent)]
    FailedConstraints(#[from] ConstraintsSolverError),
}

#[derive(Debug, Error)]
pub enum CompoundInferError<E> {
    #[error(transparent)]
    AlgoW(#[from] AlgorithmWError),
    Both(AlgorithmWError, NEVec<E>),
    Foreign(NEVec<E>)
}

struct AlgorithmW<'e> {
    monomorphic_set: HashSet<TVar>,
    env: &'e mut InferState,
}

type AWResult = Result<(AssumptionSet, Vec<Constraint>, MonomorphicType), AlgorithmWError>;

impl<'e> AlgorithmW<'e> {
    fn apply(&mut self, expression: &Language) -> AWResult {
        match expression {
            Language::Var(x) => self.apply_var(x),
            Language::App(x) => self.apply_app(x),
            Language::Lambda(x) => self.apply_lambda(x),
            Language::Let(x) => self.apply_let(x),
            Language::Special(x) => self.apply_special(x),
            Language::Literal(x) => match x {
                Literal::Integral => Ok((
                    AssumptionSet::empty(),
                    vec![],
                    PrimitiveType::Integral.into(),
                )),
                Literal::Floating => Ok((
                    AssumptionSet::empty(),
                    vec![],
                    PrimitiveType::Floating.into(),
                )),
                Literal::Tuple(vec) => self.apply_tuple(vec),
            },
        }
    }

    fn apply_var(&mut self, var: &Var) -> AWResult {
        let fresh = self.env.new_var();
        Ok((
            AssumptionSet::single(var.clone(), fresh),
            vec![],
            fresh.into(),
        ))
    }

    fn apply_app(&mut self, language::App { arg, func }: &language::App) -> AWResult {
        let (as1, cs1, t1) = self.apply(func)?;
        let (as2, cs2, t2) = self.apply(arg)?;
        let tv = self.env.new_var();

        Ok((
            as1 + as2,
            concat([cs1, cs2, vec![eq_cst(t1, fun1(t2, tv))]]),
            tv.into(),
        ))
    }

    fn apply_lambda(&mut self, language::Lambda { bind, expr }: &language::Lambda) -> AWResult {
        let tv = self.env.new_var();
        self.monomorphic_set.insert(tv);
        let (as1, cs1, t1) = self.apply(expr)?;

        let mut as_ = as1.clone();
        as_.remove(&bind.var);
        let eq_cs = as1
            .get(&bind.var)
            .iter()
            .map(|it| eq_cst(tv, it.clone()))
            .collect();
        let bound = bind
            .ty
            .as_ref()
            .map_or(vec![], |it| vec![eq_cst(tv, it.clone())]);

        Ok((as_, concat([cs1, eq_cs, bound]), fun1(tv, t1)))
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

        let mut as_ = as1.clone() + &as2;
        as_.remove(&bind.var);
        let im_cs = as2
            .get(&bind.var)
            .iter()
            .chain(as1.get(&bind.var).iter()) // support for fix
            .map(|it| implicit_cst(it.clone(), self.monomorphic_set.clone(), t1.clone()))
            .collect();
        let bound = bind.ty.as_ref().map_or(vec![], |it| {
            vec![implicit_cst(
                it.clone(),
                self.monomorphic_set.clone(),
                t1.clone(),
            )]
        });

        Ok((as_, concat([cs1, cs2, im_cs, bound]), t2))
    }

    fn apply_tuple(&mut self, tuple: &[Language]) -> AWResult {
        let ctx: Vec<_> = tuple.iter().map(|it| self.apply(it)).try_collect()?;
        let (a, c, t): (Vec<_>, Vec<_>, Vec<_>) = ctx.into_iter().multiunzip();
        Ok((
            AssumptionSet::merge_many(a),
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
    fn infer_w<E>(
        &self,
        context: &mut AlgorithmW,
        table: &impl EnvironmentProvider<Var, Error = E>,
    ) -> Result<(Substitutions, MonomorphicType), CompoundInferError<E>> {
        let (a, c, t) = context.apply(self)?;
        let (errors, not_found, explicits) = a.keys().fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut errors, mut not_found, mut cst), next| {
                match table.maybe_get(next) {
                    Ok(None) => not_found.push(next.clone()),
                    Ok(Some(s)) => cst.extend(
                        a.get(next)
                            .iter()
                            .map(|it| explicit_cst(it.clone(), s.clone().into_owned())),
                    ),
                    Err(e) => errors.push(e),
                }
                (errors, not_found, cst)
            },
        );

        if let Some(not_found) = NEVec::from_vec(not_found) {
            if let Some(errors) = NEVec::from_vec(errors) {
                return Err(CompoundInferError::Both(UnknownVar(not_found), errors))
            }
            return Err(CompoundInferError::AlgoW(UnknownVar(not_found)))
        } else if let Some(errors) = NEVec::from_vec(errors) {
            return Err(CompoundInferError::Foreign(errors))
        }

        debug!("Inferred raw type and constraints: ");
        debug!("{c:?} ++ {explicits:?}, {t}");
        let substitutions = Constraint::solve(concat([c, explicits]), context.env)
            .map_err(AlgorithmWError::FailedConstraints)?;
        let t = t.substitute(&substitutions);
        debug!("Inferred type and substitutions: ");
        debug!("{}, {}", substitutions, t);
        Ok((substitutions, t))
    }

    pub(crate) fn infer_with_env<E>(
        &self,
        context: &impl EnvironmentProvider<Var, Error = E>,
        env: &mut InferState,
    ) -> Result<PolymorphicType, CompoundInferError<E>> {
        let mut ctx = AlgorithmW {
            monomorphic_set: Default::default(),
            env,
        };
        match self.infer_w(&mut ctx, context) {
            Ok((s, t)) => Ok(t.substitute(&s).normalize()),
            Err(e) => Err(e),
        }
    }

    pub fn infer<E>(
        &self,
        table: &impl EnvironmentProvider<Var, Error = E>,
    ) -> Result<PolymorphicType, CompoundInferError<E>> {
        self.infer_with_env(table, &mut InferState::default())
    }
}

impl Display for AlgorithmWError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgorithmWError::AlgorithmU(x) => write!(f, "{x}"),
            UnknownVar(vs) => write!(
                f,
                "Unknown references: [{}]",
                vs.iter().into_iter().join(", ")
            ),
            AlgorithmWError::FailedConstraints(x) => write!(f, "{x}"),
        }
    }
}

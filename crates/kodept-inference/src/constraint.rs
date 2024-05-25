use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::iter;

use derive_more::Display;
use itertools::Itertools;
use thiserror::Error;

use Constraint::{ExplicitInstance, ImplicitInstance};
use ConstraintsSolverError::AlgorithmU;

use crate::algorithm_u::AlgorithmUError;
use crate::constraint::Constraint::Eq;
use crate::constraint::ConstraintsSolverError::Ambiguous;
use crate::Environment;
use crate::r#type::{MonomorphicType, PolymorphicType, TVar};
use crate::substitution::Substitutions;
use crate::traits::{ActiveTVars, FreeTypeVars, Substitutable};

#[derive(Debug, Error)]
pub enum ConstraintsSolverError {
    #[error(transparent)]
    AlgorithmU(#[from] AlgorithmUError),
    Ambiguous(Vec<Constraint>),
}

#[derive(Debug, PartialEq, Clone, Display)]
#[display(fmt = "{t1} ≡ {t2}")]
pub struct EqConstraint {
    pub t1: MonomorphicType,
    pub t2: MonomorphicType,
}

/// Types of constraints used in algorithm W
#[derive(Debug, PartialEq, Clone)]
pub enum Constraint {
    /// t1 should be unified with t2
    Eq(EqConstraint),
    /// t should be an instance of s
    ExplicitInstance {
        t: MonomorphicType,
        s: PolymorphicType,
    },
    /// t1 should be an instance of generalize(t2, ctx)
    ImplicitInstance {
        t1: MonomorphicType,
        ctx: HashSet<TVar>,
        t2: MonomorphicType,
    },
}

type Pair<'a> = (&'a Constraint, Vec<&'a Constraint>);

impl Display for ConstraintsSolverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgorithmU(x) => write!(f, "{x}")?,
            Ambiguous(x) => {
                for item in x {
                    match item {
                        Eq(EqConstraint { t1, t2 }) => {
                            write!(
                                f,
                                "Cannot match expected type `{t1}` with actual type `{t2}`"
                            )?;
                        }
                        ExplicitInstance { t, s } => {
                            write!(f, "Cannot match instance `{t}` of type `{s}`")?;
                        }
                        ImplicitInstance { t1, ctx, t2 } => {
                            write!(f, "Cannot match expected type `{t1}` with generalization of type `{t2}` in context {{{}}}", ctx.iter().join(", "))?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Constraint {
    fn pairs(vec: &[Constraint]) -> impl Iterator<Item = Pair> {
        let mut idx = 0;

        iter::from_fn(move || {
            if vec.len() <= idx {
                return None;
            }
            let mut copy = Vec::from_iter(vec);
            let item = copy.swap_remove(idx);
            idx += 1;
            Some((item, copy))
        })
    }

    fn solvable(pair: &Pair) -> bool {
        match pair {
            (Eq(EqConstraint { .. }), _) => true,
            (ExplicitInstance { .. }, _) => true,
            (ImplicitInstance { ctx, t2, .. }, cs) => {
                let v1 = &t2.free_types() - ctx;
                let active = cs.iter().map(|it| *it).active_vars();
                (&v1 & &active).is_empty()
            }
        }
    }

    fn solve_pair(
        (c, cs): Pair,
        env: &mut Environment,
    ) -> Result<Substitutions, ConstraintsSolverError> {
        match c {
            Eq(EqConstraint { t1, t2 }) => {
                let s1 = t1.unify(t2)?;
                let s2 = Self::solve(&cs.substitute(&s1), env)?;
                Ok(s2 + s1)
            }
            ExplicitInstance { t, s } => {
                let t2 = s.instantiate(env);
                let mut cs: Vec<_> = cs.into_iter().cloned().collect();
                cs.insert(0, Eq(EqConstraint { t1: t.clone(), t2 }));
                Self::solve(&cs, env)
            }
            ImplicitInstance { t1, ctx, t2 } => {
                let s = t2.generalize(ctx);
                let mut cs: Vec<_> = cs.into_iter().cloned().collect();
                cs.insert(0, ExplicitInstance { t: t1.clone(), s });
                Self::solve(&cs, env)
            }
        }
    }

    pub fn solve(
        iter: &[Constraint],
        env: &mut Environment,
    ) -> Result<Substitutions, ConstraintsSolverError> {
        if iter.is_empty() {
            return Ok(Substitutions::empty());
        }
        if let Some(next) = Self::pairs(iter).find(Self::solvable) {
            Self::solve_pair(next, env)
        } else {
            Err(Ambiguous(iter.to_vec()))
        }
    }
}

impl Display for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Eq(x) => write!(f, "{x}"),
            ExplicitInstance { t, s } => write!(f, "{t} ≼ {s}"),
            ImplicitInstance { t1, ctx, t2 } => {
                write!(f, "{t1} ≤{{{}}} {t2}", ctx.iter().join(", "))
            }
        }
    }
}

pub fn eq_cst(t1: impl Into<MonomorphicType>, t2: impl Into<MonomorphicType>) -> Constraint {
    Eq(EqConstraint {
        t1: t1.into(),
        t2: t2.into(),
    })
}

pub fn implicit_cst(
    t1: impl Into<MonomorphicType>,
    ctx: HashSet<TVar>,
    t2: impl Into<MonomorphicType>,
) -> Constraint {
    ImplicitInstance {
        t1: t1.into(),
        ctx,
        t2: t2.into(),
    }
}

pub fn explicit_cst(t: impl Into<MonomorphicType>, s: impl Into<PolymorphicType>) -> Constraint {
    ExplicitInstance {
        t: t.into(),
        s: s.into(),
    }
}

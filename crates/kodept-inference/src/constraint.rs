use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Formatter};

use derive_more::Display;
use itertools::Either::{Left, Right};
use itertools::{Either, Itertools};
use thiserror::Error;

use Constraint::{ExplicitInstance, ImplicitInstance};
use ConstraintsSolverError::AlgorithmU;

use crate::algorithm_u::AlgorithmUError;
use crate::constraint::Constraint::Eq;
use crate::constraint::ConstraintsSolverError::Ambiguous;
use crate::r#type::{MonomorphicType, PolymorphicType, TVar};
use crate::substitution::Substitutions;
use crate::traits::{ActiveTVars, FreeTypeVars, Substitutable};
use crate::InferState;

#[derive(Debug, Error)]
pub enum ConstraintsSolverError {
    #[error(transparent)]
    AlgorithmU(#[from] AlgorithmUError),
    Ambiguous(Vec<Constraint>),
}

#[derive(Debug, PartialEq, Clone, Display)]
#[display("{t1} ≡ {t2}")]
pub struct EqConstraint {
    pub t1: MonomorphicType,
    pub t2: MonomorphicType,
}

/// Types of constraints used in algorithm W
#[derive(PartialEq, Clone)]
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
    fn solvable(c: &Constraint, cs: &VecDeque<Constraint>) -> bool {
        match c {
            Eq(EqConstraint { .. }) => true,
            ExplicitInstance { .. } => true,
            ImplicitInstance { ctx, t2, .. } => {
                let v1 = &t2.free_types() - ctx;
                let active = cs.active_vars();
                (&v1 & &active).is_empty()
            }
        }
    }

    fn solve_pair(
        c: Constraint,
        env: &mut InferState,
    ) -> Result<Either<Substitutions, Constraint>, ConstraintsSolverError> {
        match c {
            Eq(EqConstraint { t1, t2 }) => Ok(Left(t1.unify(&t2)?)),
            ExplicitInstance { t, s } => {
                let t2 = s.instantiate(env);
                Ok(Right(Eq(EqConstraint { t1: t, t2 })))
            }
            ImplicitInstance { t1, ctx, t2 } => {
                let s = t2.generalize(&ctx);
                Ok(Right(ExplicitInstance { t: t1, s }))
            }
        }
    }

    pub(crate) fn solve(
        constraints: Vec<Constraint>,
        env: &mut InferState,
    ) -> Result<Substitutions, ConstraintsSolverError> {
        let mut cs = VecDeque::from(constraints);
        let mut s0 = Substitutions::empty();

        // solver should always find suitable constraint to solve
        while let Some(c) = cs.pop_back() {
            if Self::solvable(&c, &cs) {
                match Self::solve_pair(c, env)? {
                    Left(s) => {
                        cs = cs.make_contiguous().substitute(&s).into();
                        s0 = s0 + s;
                    }
                    Right(c) => cs.push_front(c),
                }
            } else {
                cs.push_front(c)
            }
        }
        Ok(s0)
    }
}

impl Display for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Eq(x) => write!(f, "{x}"),
            ExplicitInstance { t, s } => write!(f, "{t} ≼ ({s})"),
            ImplicitInstance { t1, ctx, t2 } => {
                write!(f, "{t1} ≤{{{}}} {t2}", ctx.iter().join(", "))
            }
        }
    }
}

impl Debug for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
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
    ctx: impl Into<HashSet<TVar>>,
    t2: impl Into<MonomorphicType>,
) -> Constraint {
    ImplicitInstance {
        t1: t1.into(),
        ctx: ctx.into(),
        t2: t2.into(),
    }
}

pub fn explicit_cst(t: impl Into<MonomorphicType>, s: impl Into<PolymorphicType>) -> Constraint {
    ExplicitInstance {
        t: t.into(),
        s: s.into(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::constraint::{eq_cst, implicit_cst, Constraint};
    use crate::r#type::MonomorphicType::Var;
    use crate::r#type::PrimitiveType::Boolean;
    use crate::r#type::{fun1, TVar};
    use crate::substitution::Substitutions;
    use crate::InferState;

    #[test]
    fn test_1() {
        let mut env = InferState::default();
        let [t1, t2, t3, t4, t5] = [1, 2, 3, 4, 5].map(TVar);
        env.variable_index = 6;
        let cs = vec![
            eq_cst(t2, fun1(Boolean, t3)),
            implicit_cst(t4, [t5], t3),
            implicit_cst(t2, [t5], t1),
            eq_cst(t5, t1),
        ];

        let result = Constraint::solve(cs, &mut env).unwrap();
        assert_eq!(
            result,
            Substitutions::from_iter([
                (t4, Var(t3)),
                (t1, fun1(Boolean, t3)),
                (t5, fun1(Boolean, t3)),
                (t2, fun1(Boolean, t3))
            ])
        )
    }
    
    #[test]
    fn test_2() {
        let mut env = InferState::default();
        let [t0, t1, t2, t3, t4] = [0, 1, 2, 3, 4].map(TVar);
        env.variable_index = 5;
        
        let cs = vec![
            eq_cst(t1, fun1(t2, t3)),
            implicit_cst(t4, [t0], t3),
            implicit_cst(t2, [t0], t3),
            eq_cst(t0, t1)
        ];
        
        let result = Constraint::solve(cs, &mut env).unwrap();
        assert_eq!(result, Substitutions::from_iter([
            (t0, fun1(t3, t3)),
            (t2, t3.into()),
            (t4, t3.into()),
            (t1, fun1(t3, t3))
        ]));
    }
}

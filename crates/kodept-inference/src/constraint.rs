use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Display, Formatter};

use derive_more::{Display, Error, From};

use crate::algorithm_u::AlgorithmUError;
use crate::constraint::Constraint::Eq;
use crate::constraint::ConstraintsSolverError::{AlgorithmU, Ambiguous};
use crate::constraint::Either::{Left, Right};
use crate::substitution::Substitutions;
use crate::traits::{ActiveTVars, FreeTypeVars, Substitutable};
use crate::r#type::{MonomorphicType, PolymorphicType, TVar};
use crate::utils::JoinedDisplay;
use Constraint::{ExplicitInstance, ImplicitInstance};
use kodept_interning::{InternInto, Interned};

#[derive(Debug, Error, From)]
pub enum ConstraintsSolverError {
    AlgorithmU(AlgorithmUError),
    #[from(ignore)]
    Ambiguous(#[error(not(source))] Vec<Constraint>),
}

#[derive(Debug)]
enum Either<A, B> {
    Left(A),
    Right(B),
}

#[derive(Debug, PartialEq, Clone, Display)]
#[display("{t1} ≡ {t2}")]
pub struct EqConstraint {
    pub t1: Interned<MonomorphicType>,
    pub t2: Interned<MonomorphicType>,
}

/// Types of constraints used in algorithm W
#[derive(PartialEq, Clone)]
pub enum Constraint {
    /// t1 should be unified with t2
    Eq(EqConstraint),
    /// t should be an instance of s
    ExplicitInstance {
        t: Interned<MonomorphicType>,
        s: PolymorphicType,
    },
    /// t1 should be an instance of generalize(t2, ctx)
    ImplicitInstance {
        t1: Interned<MonomorphicType>,
        ctx: HashSet<TVar>,
        t2: Interned<MonomorphicType>,
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
                            write!(
                                f,
                                "Cannot match expected type `{t1}` with generalization of type `{t2}` in context {{{}}}",
                                JoinedDisplay::enumerate(ctx)
                            )?;
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
    ) -> Result<Either<Substitutions, Constraint>, ConstraintsSolverError> {
        match c {
            Eq(EqConstraint { t1, t2 }) => Ok(Left(t1.unify(&t2)?)),
            ExplicitInstance { t, s } => {
                let t2 = s.instantiate();
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
    ) -> Result<Substitutions, ConstraintsSolverError> {
        let mut cs = VecDeque::from(constraints);
        let mut s0 = Substitutions::empty();

        // solver should always find suitable constraint to solve
        while let Some(c) = cs.pop_back() {
            if Self::solvable(&c, &cs) {
                match Self::solve_pair(c)? {
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
            ExplicitInstance { t, s } => write!(f, "{t} ≼ {s}"),
            ImplicitInstance { t1, ctx, t2 } => {
                write!(f, "{t1} ≤{{{}}} {t2}", JoinedDisplay::enumerate(ctx))
            }
        }
    }
}

impl Debug for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub fn eq_cst(t1: impl InternInto<MonomorphicType>, t2: impl InternInto<MonomorphicType>) -> Constraint {
    Eq(EqConstraint {
        t1: t1.intern_into(),
        t2: t2.intern_into(),
    })
}

pub fn implicit_cst(
    t1: impl InternInto<MonomorphicType>,
    ctx: impl Into<HashSet<TVar>>,
    t2: impl InternInto<MonomorphicType>,
) -> Constraint {
    ImplicitInstance {
        t1: t1.intern_into(),
        ctx: ctx.into(),
        t2: t2.intern_into(),
    }
}

pub fn explicit_cst(t: impl InternInto<MonomorphicType>, s: impl Into<PolymorphicType>) -> Constraint {
    ExplicitInstance {
        t: t.intern_into(),
        s: s.into(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::constraint::{Constraint, eq_cst, implicit_cst};
    use crate::substitution::Substitutions;
    use crate::r#type::MonomorphicType::Var;
    use crate::r#type::PrimitiveType::Boolean;
    use crate::r#type::{MonomorphicType, TVar};
    use kodept_interning::GlobalInterner;

    #[test]
    fn test_1() {
        let [t1, t2, t3, t4, t5] = [1, 2, 3, 4, 5].map(|_| TVar::new());
        let cs = vec![
            eq_cst(
                &Var(t2),
                &MonomorphicType::fun1(&Boolean.intern().into(), &Var(t3)),
            ),
            implicit_cst(&Var(t4), [t5], &Var(t3)),
            implicit_cst(&Var(t2), [t5], &Var(t1)),
            eq_cst(&Var(t5), &Var(t1)),
        ];

        let result = Constraint::solve(cs).unwrap();
        assert_eq!(
            result,
            Substitutions::from_iter([
                (t4, Var(t3)),
                (
                    t1,
                    MonomorphicType::fun1(&Boolean.intern().into(), &Var(t3))
                ),
                (
                    t5,
                    MonomorphicType::fun1(&Boolean.intern().into(), &Var(t3))
                ),
                (
                    t2,
                    MonomorphicType::fun1(&Boolean.intern().into(), &Var(t3))
                )
            ])
        )
    }

    #[test]
    fn test_2() {
        let [t0, t1, t2, t3, t4] = [0, 1, 2, 3, 4].map(|_| TVar::new());

        let cs = vec![
            eq_cst(&Var(t1), &MonomorphicType::fun1(&Var(t2), &Var(t3))),
            implicit_cst(&Var(t4), [t0], &Var(t3)),
            implicit_cst(&Var(t2), [t0], &Var(t3)),
            eq_cst(&Var(t0), &Var(t1)),
        ];

        let result = Constraint::solve(cs).unwrap();
        assert_eq!(
            result,
            Substitutions::from_iter([
                (t0, MonomorphicType::fun1(&Var(t3), &Var(t3))),
                (t2, t3.into()),
                (t4, t3.into()),
                (t1, MonomorphicType::fun1(&Var(t3), &Var(t3)))
            ])
        );
    }
}

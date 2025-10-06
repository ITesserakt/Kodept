use derive_more::{Error, From};
use itertools::{concat, Itertools};
use nonempty_collections::NEVec;
use std::collections::HashSet;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use tracing::debug;

use crate::algorithm_u::AlgorithmUError;
use crate::algorithm_w::AlgorithmWError::UnknownVar;
use crate::constraint::{eq_cst, explicit_cst, implicit_cst, Constraint, ConstraintsSolverError};
use crate::process::{Infer, PartialInfer};
use crate::r#type::PrimitiveType::Boolean;
use crate::r#type::{fun1, unit_type, MonomorphicType, PolymorphicType, PrimitiveType, TVar};
use crate::substitution::Substitutions;
use crate::traits::{EnvironmentProvider, Substitutable, TypeInfer};

#[derive(Debug, Error, From)]
pub enum AlgorithmWError {
    AlgorithmU(AlgorithmUError),
    #[from(ignore)]
    UnknownVar(#[error(not(source))] NEVec<Var>),
    FailedConstraints(ConstraintsSolverError),
}

#[derive(Debug, Error, From)]
pub enum CompoundInferError<E> {
    AlgoW(AlgorithmWError),
    #[from(ignore)]
    Both(AlgorithmWError, NEVec<E>),
    #[from(ignore)]
    Foreign(NEVec<E>),
}

#[derive(Debug, Clone, Default)]
struct AlgorithmW {
    monomorphic_set: HashSet<TVar>,
    env: InferState,
}

impl AlgorithmW {
    fn apply_(&mut self, expression: &Language) -> PartialInfer {
        match expression {
            Language::Var(x) => self.apply_var(x),
            Language::App(x) => self.apply_app(x),
            Language::Lambda(x) => self.apply_lambda(x),
            Language::Let(x) => self.apply_let(x),
            Language::Special(x) => self.apply_special(x),
            Language::Literal(x) => match x {
                Literal::Integral => {
                    PartialInfer::new(AssumptionSet::empty(), [None], PrimitiveType::i8())
                }
                Literal::Floating => {
                    PartialInfer::new(AssumptionSet::empty(), [None], PrimitiveType::f24())
                }
                Literal::Tuple(vec) => self.apply_tuple(vec),
            },
        }
    }

    fn apply_var(&mut self, var: &Var) -> PartialInfer {
        let fresh = self.env.new_var();
        PartialInfer::new(AssumptionSet::single(var.clone(), fresh), [None], fresh)
    }

    fn apply_app(&mut self, language::App { arg, func }: &language::App) -> PartialInfer {
        let PartialInfer(as1, cs1, t1) = self.apply_(func);
        let PartialInfer(as2, cs2, t2) = self.apply_(arg);
        let tv = self.env.new_var();

        PartialInfer::new(as1 + as2, [cs1, cs2, vec![eq_cst(t1, fun1(t2, tv))]], tv)
    }

    fn apply_lambda(&mut self, language::Lambda { bind, expr }: &language::Lambda) -> PartialInfer {
        let tv = self.env.new_var();
        self.monomorphic_set.insert(tv);
        let PartialInfer(as1, cs1, t1) = self.apply_(expr);

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

        PartialInfer::new(as_, [cs1, eq_cs, bound], fun1(tv, t1))
    }

    fn apply_let(
        &mut self,
        language::Let {
            binder,
            bind,
            usage,
        }: &language::Let,
    ) -> PartialInfer {
        let PartialInfer(as1, cs1, t1) = self.apply_(binder);
        let PartialInfer(as2, cs2, t2) = self.apply_(usage);

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

        PartialInfer::new(as_, [cs1, cs2, im_cs, bound], t2)
    }

    fn apply_tuple(&mut self, tuple: &[Language]) -> PartialInfer {
        let mut assumptions = AssumptionSet::empty();
        let mut constraints = vec![];

        let items = tuple
            .iter()
            .map(|it| {
                let PartialInfer(a, c, t) = self.apply_(it);
                assumptions.merge(a);
                constraints.extend(c);
                t
            })
            .collect();

        PartialInfer(assumptions, constraints, MonomorphicType::Tuple(items))
    }

    fn apply_special(&mut self, special: &Special) -> PartialInfer {
        match special {
            Special::If {
                condition,
                body,
                otherwise,
            } => {
                let PartialInfer(as1, cs1, t1) = self.apply_(condition);
                let PartialInfer(as2, cs2, t2) = self.apply_(body);
                let PartialInfer(as3, cs3, t3) = self.apply_(otherwise);

                PartialInfer::new(
                    as1 + as2 + as3,
                    [
                        cs1,
                        cs2,
                        cs3,
                        vec![eq_cst(t1, Boolean), eq_cst(t2.clone(), t3)],
                    ],
                    t2,
                )
            }
        }
    }
}

impl<'a> TypeInfer<&'a Language> for AlgorithmW {
    type Error = Infallible;
    type Output = PartialInfer;

    fn apply<'b>(&mut self, expr: &'a Language) -> Infer<'b, &'a Language, Self>
    where
        &'a Language: 'b,
    {
        match expr {
            Language::Var(x) => {
                let fresh = self.env.new_var();
                Infer::done(PartialInfer::new(
                    AssumptionSet::single(x.clone(), fresh),
                    [None],
                    fresh,
                ))
            }
            Language::App(language::App { arg, func }) => {
                Self::suspend(func).and_then(|_, PartialInfer(a1, c1, t1)| {
                    Self::suspend(arg).map(|state, PartialInfer(a2, c2, t2)| {
                        let fresh = state.env.new_var();
                        PartialInfer::new(
                            a1 + a2,
                            [c1, c2, vec![eq_cst(t1, fun1(t2, fresh))]],
                            fresh,
                        )
                    })
                })
            }
            Language::Lambda(language::Lambda { bind, expr }) => {
                let tv = self.env.new_var();
                self.monomorphic_set.insert(tv);
                Self::suspend(expr).map(move |_, PartialInfer(as1, cs1, t1)| {
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
                    PartialInfer::new(as_, [cs1, eq_cs, bound], fun1(tv, t1))
                })
            }
            Language::Let(language::Let {
                binder,
                bind,
                usage,
            }) => {
                Self::suspend(binder).and_then(move |_, PartialInfer(as1, cs1, t1)| {
                    Self::suspend(usage).map(move |state, PartialInfer(as2, cs2, t2)| {
                        let mut as_ = as1.clone() + &as2;
                        as_.remove(&bind.var);
                        let im_cs = as2
                            .get(&bind.var)
                            .iter()
                            .chain(as1.get(&bind.var).iter()) // support for fix
                            .map(|it| {
                                implicit_cst(it.clone(), state.monomorphic_set.clone(), t1.clone())
                            })
                            .collect();
                        let bound = bind.ty.as_ref().map_or(vec![], |it| {
                            vec![implicit_cst(
                                it.clone(),
                                state.monomorphic_set.clone(),
                                t1.clone(),
                            )]
                        });
                        PartialInfer::new(as_, [cs1, cs2, im_cs, bound], t2)
                    })
                })
            }
            Language::Literal(Literal::Integral) => Infer::done(PartialInfer::new(
                AssumptionSet::empty(),
                [None],
                PrimitiveType::i8(),
            )),
            Language::Literal(Literal::Floating) => Infer::done(PartialInfer::new(
                AssumptionSet::empty(),
                [None],
                PrimitiveType::f24(),
            )),
            Language::Literal(Literal::Tuple(items)) => {
                let current = Infer::done(PartialInfer::new(
                    AssumptionSet::empty(),
                    [None],
                    unit_type(),
                ));
                // Very bad way of doing things incoming
                items.iter().fold(current, |acc, next| {
                    acc.zip_with(Self::infer(next), |mut a, b| {
                        a.0.merge(b.0);
                        a.1.extend(b.1);
                        if let MonomorphicType::Tuple(vec) = a.2 {
                            let mut vec = vec.to_vec();
                            vec.push(b.2);
                            a.2 = MonomorphicType::Tuple(vec.into());
                        }
                        a
                    })
                })
            }
            Language::Special(Special::If {
                condition,
                body,
                otherwise,
            }) => Self::suspend(condition).and_then(move |_, PartialInfer(as1, c1, t1)| {
                Self::suspend(body).and_then(move |_, PartialInfer(as2, c2, t2)| {
                    Self::suspend(otherwise).map(move |_, PartialInfer(as3, c3, t3)| {
                        PartialInfer::new(
                            as1 + as2 + as3,
                            [
                                c1,
                                c2,
                                c3,
                                vec![eq_cst(t1, Boolean), eq_cst(t2.clone(), t3)],
                            ],
                            t2,
                        )
                    })
                })
            }),
        }
    }
}

impl Language {
    fn infer_w<E>(
        &self,
        context: &mut AlgorithmW,
        table: &impl EnvironmentProvider<Var, Error = E>,
    ) -> Result<(Substitutions, MonomorphicType), CompoundInferError<E>> {
        let PartialInfer(a, c, t) = context.apply_(self);
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

        if let Some(not_found) = NEVec::try_from_vec(not_found) {
            if let Some(errors) = NEVec::try_from_vec(errors) {
                return Err(CompoundInferError::Both(UnknownVar(not_found), errors));
            }
            return Err(CompoundInferError::AlgoW(UnknownVar(not_found)));
        } else if let Some(errors) = NEVec::try_from_vec(errors) {
            return Err(CompoundInferError::Foreign(errors));
        }

        debug!("Inferred raw type and constraints: ");
        debug!("{c:?} ++ {explicits:?}, {t}");
        let substitutions = Constraint::solve(concat([c, explicits]), &mut context.env)
            .map_err(AlgorithmWError::FailedConstraints)?;
        let t = t.substitute(&substitutions);
        debug!("Inferred type and substitutions: ");
        debug!("{}, {}", substitutions, t);
        Ok((substitutions, t))
    }

    pub(crate) fn infer_with_env<E>(
        &self,
        context: &impl EnvironmentProvider<Var, Error = E>,
        env: InferState,
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
        self.infer_with_env(table, InferState::default())
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

#[cfg(test)]
mod tests {
    use crate::algorithm_w::AlgorithmW;
    use crate::language::Language;
    use crate::process::DefaultExecutor;
    use crate::traits::TypeInfer;
    use proptest::{prop_assert_eq, proptest};

    proptest! {
        #[test]
        fn proptest_infer_algorithms(expr: Language) {
            let result = AlgorithmW::default().infer_eagerly(&expr, &mut DefaultExecutor::default())?;
            let other_result = AlgorithmW::default().apply_(&expr);

            prop_assert_eq!(result, other_result);
        }
    }
}

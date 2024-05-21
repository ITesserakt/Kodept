use MonomorphicType::*;

use crate::r#type::{MonomorphicType, Tuple, TVar};
use crate::substitution::Substitutions;

#[derive(Debug)]
pub enum AlgorithmUError {
    InfiniteType {
        type_var: TVar,
        with: MonomorphicType,
    },
    CannotUnify {
        from: MonomorphicType,
        to: MonomorphicType,
    },
}

struct AlgorithmU;

impl AlgorithmU {
    fn occurs_check(var: &TVar, with: &MonomorphicType) -> bool {
        match (var, with) {
            (TVar(a), MonomorphicType::Var(TVar(b))) if *a == *b => true,
            (var, Fn { input, output }) => {
                AlgorithmU::occurs_check(var, input) || AlgorithmU::occurs_check(var, output)
            }
            (var, Pointer(t)) => AlgorithmU::occurs_check(var, t),
            (var, MonomorphicType::Tuple(Tuple(vec))) => {
                vec.iter().any(|it| AlgorithmU::occurs_check(var, it))
            }
            _ => false,
        }
    }

    fn unify_vec(
        vec1: &[MonomorphicType],
        vec2: &[MonomorphicType],
    ) -> Result<Substitutions, AlgorithmUError> {
        vec1.iter()
            .zip(vec2.iter())
            .try_fold(Substitutions::empty(), |acc, (x, y)| {
                x.substitute(&acc)
                    .unify(&y.substitute(&acc))
                    .map(|it| acc.compose(&it))
            })
    }

    fn apply(
        lhs: &MonomorphicType,
        rhs: &MonomorphicType,
    ) -> Result<Substitutions, AlgorithmUError> {
        match (lhs, rhs) {
            (a, b) if a == b => Ok(Substitutions::empty()),
            (MonomorphicType::Var(var), b) if AlgorithmU::occurs_check(var, b) => {
                Err(AlgorithmUError::InfiniteType {
                    type_var: var.clone(),
                    with: b.clone(),
                })
            }
            (a, MonomorphicType::Var(var)) if AlgorithmU::occurs_check(var, a) => {
                Err(AlgorithmUError::InfiniteType {
                    type_var: var.clone(),
                    with: a.clone(),
                })
            }
            (a @ MonomorphicType::Var(_), b) => Ok(Substitutions::single(b.clone(), a.clone())),
            (a, b @ MonomorphicType::Var(_)) => Ok(Substitutions::single(a.clone(), b.clone())),
            (
                Fn {
                    input: input1,
                    output: output1,
                },
                Fn { input, output },
            ) => {
                let s1 = input1.unify(input)?;
                let s2 = output1.substitute(&s1).unify(&output.substitute(&s1))?;
                Ok(s2.compose(&s1))
            }
            (Pointer(p1), Pointer(p2)) => p1.unify(p2),
            (MonomorphicType::Tuple(vec1), MonomorphicType::Tuple(vec2))
                if vec1.0.len() == vec2.0.len() =>
            {
                AlgorithmU::unify_vec(&vec1.0, &vec2.0)
            },
            _ => Err(AlgorithmUError::CannotUnify {
                from: lhs.clone(),
                to: rhs.clone(),
            }),
        }
    }
}

impl MonomorphicType {
    pub fn unify(&self, other: &MonomorphicType) -> Result<Substitutions, AlgorithmUError> {
        AlgorithmU::apply(self, other)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::HashSet;

    use nonempty_collections::nev;

    use crate::algorithm_u::AlgorithmUError;
    use crate::r#type::{fun, fun1, PrimitiveType, Tuple, var};
    use crate::r#type::MonomorphicType::Constant;
    use crate::substitution::{Substitution, Substitutions};

    #[test]
    fn test_tautology_example_on_constants() {
        let a = Constant(1);
        let b = Constant(1);

        let s = a.unify(&b).unwrap();
        assert_eq!(s.0, HashSet::new());
    }

    #[test]
    fn test_different_constants_should_not_unify() {
        let a = Constant(1);
        let b = Constant(2);

        let e = a.unify(&b).unwrap_err();
        assert!(matches!(e, AlgorithmUError::CannotUnify { .. }))
    }

    #[test]
    fn test_tautology_example_on_vars() {
        let a = var(0);
        let b = var(0);

        let s = a.unify(&b).unwrap();
        assert_eq!(s.0, HashSet::new());
    }

    #[test]
    fn test_variables_should_be_always_unified() {
        let a = var(1);
        let b = Constant(1);

        let s1 = a.unify(&b).unwrap();
        let s2 = b.unify(&a).unwrap();

        assert_eq!(s1, s2);
        assert_eq!(s1, Substitutions::single(b, a))
    }

    #[test]
    fn test_aliasing() {
        let a = var(1);
        let b = var(2);

        let s1 = a.unify(&b).unwrap();
        let s2 = b.unify(&a).unwrap();

        assert_eq!(s1, Substitutions::single(b.clone(), a.clone()));
        assert_eq!(s2, Substitutions::single(a, b))
    }

    #[test]
    fn test_simple_function_unifying() {
        let a = fun(nev![var(1), Constant(0)], Tuple::unit());
        let b = fun(nev![var(1), var(2)], Tuple::unit());

        let s = a.unify(&b).unwrap();
        assert_eq!(s, Substitutions::single(Constant(0), var(2)));
    }

    #[test]
    fn test_aliasing_in_functions() {
        let a = fun(nev![var(1)], Tuple::unit());
        let b = fun(nev![var(2)], Tuple::unit());

        let s = a.unify(&b).unwrap();
        assert_eq!(s, Substitutions::single(var(2), var(1)));
    }

    #[test]
    fn test_functions_with_different_arity_should_not_unify() {
        let a = fun(nev![Constant(1)], Tuple::unit());
        let b = fun(nev![Constant(1), Constant(2)], Tuple::unit());

        let s = a.unify(&b).unwrap_err();
        assert!(matches!(s, AlgorithmUError::CannotUnify { .. }))
    }

    #[test]
    fn test_multiple_substitutions() {
        let a = fun(
            nev![fun1(var(1), PrimitiveType::Integral), var(1)],
            Tuple::unit(),
        );
        let b = fun(nev![var(2), Constant(0)], Tuple::unit());

        let s = a.unify(&b).unwrap();
        assert_eq!(
            s.0,
            HashSet::from([
                Substitution::new(Constant(0), var(1)),
                Substitution::new(fun1(Constant(0), PrimitiveType::Integral), var(2))
            ])
        )
    }

    #[test]
    fn test_infinite_substitution() {
        let a = var(1);
        let b = fun1(var(1), Tuple::unit());

        let e = a.unify(&b).unwrap_err();
        assert!(matches!(e, AlgorithmUError::InfiniteType { .. }))
    }

    #[test]
    fn test_transitive_substitutions() {
        let a = var(1);
        let b = var(2);
        let c = Constant(0);

        let s1 = a.unify(&b).unwrap();
        let s2 = b.unify(&a).unwrap();
        let s3 = c.unify(&b.substitute(&s2)).unwrap();
        let s4 = a.substitute(&s1).unify(&c).unwrap();

        assert_eq!(s1, Substitutions::single(b.clone(), a.clone()));
        assert_eq!(s2, Substitutions::single(a.clone(), b.clone()));
        assert_eq!(s3, Substitutions::single(c.clone(), a));
        assert_eq!(s4, Substitutions::single(c, b));
    }

    #[test]
    fn test_different_substitutions_of_same_variable() {
        let a = var(1);
        let b = Constant(1);
        let c = Constant(2);

        let s = a.unify(&b).unwrap();
        let e = a.substitute(&s).unify(&c).unwrap_err();

        assert_eq!(s, Substitutions::single(b, a));
        assert!(matches!(e, AlgorithmUError::CannotUnify { .. }))
    }

    #[test]
    fn test_complex_unification() {
        let a = fun1(fun1(fun1(Constant(0), var(1)), var(2)), var(3));
        let b = fun1(var(3), fun1(var(2), fun1(var(1), Constant(0))));

        let s1 = a.unify(&b).unwrap();
        let s2 = b.unify(&a).unwrap();

        assert_eq!(s1, s2);
        assert_eq!(a.substitute(&s1), b.substitute(&s2));
    }
}

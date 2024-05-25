use std::fmt::{Display, Formatter};

use itertools::Itertools;
use thiserror::Error;

use MonomorphicType::*;

use crate::algorithm_u::AlgorithmUError::{InfiniteType, UnificationFail};
use crate::r#type::{MonomorphicType, TVar};
use crate::substitution::Substitutions;
use crate::traits::FreeTypeVars;

#[derive(Debug, Error)]
pub struct UnificationMismatch(pub Vec<MonomorphicType>, pub Vec<MonomorphicType>);

#[derive(Debug, Error)]
pub enum AlgorithmUError {
    #[error("Cannot unify types: {0} with {1}")]
    UnificationFail(MonomorphicType, MonomorphicType),
    #[error("Cannot construct an infinite type: {0} ~ {1}")]
    InfiniteType(TVar, MonomorphicType),
    #[error(transparent)]
    UnificationMismatch(#[from] UnificationMismatch),
}

struct AlgorithmU;

impl AlgorithmU {
    fn occurs_check(var: &TVar, with: impl FreeTypeVars) -> bool {
        with.free_types().contains(var)
    }

    fn unify_vec(
        vec1: &[MonomorphicType],
        vec2: &[MonomorphicType],
    ) -> Result<Substitutions, AlgorithmUError> {
        match (vec1, vec2) {
            ([], []) => Ok(Substitutions::empty()),
            ([t1, ts1 @ ..], [t2, ts2 @ ..]) => {
                let s1 = t1.unify(t2)?;
                let s2 = Self::unify_vec(ts1, ts2)?;
                Ok(s1 + s2)
            }
            (t1, t2) => Err(UnificationMismatch(
                t1.into_iter().cloned().collect(),
                t2.into_iter().cloned().collect(),
            )
            .into()),
        }
    }

    fn bind(var: &TVar, ty: &MonomorphicType) -> Result<Substitutions, AlgorithmUError> {
        match ty {
            Var(v) if var == v => Ok(Substitutions::empty()),
            _ if Self::occurs_check(var, ty) => Err(InfiniteType(var.clone(), ty.clone())),
            _ => Ok(Substitutions::single(var.clone(), ty.clone())),
        }
    }

    fn apply(
        lhs: &MonomorphicType,
        rhs: &MonomorphicType,
    ) -> Result<Substitutions, AlgorithmUError> {
        match (lhs, rhs) {
            (a, b) if a == b => Ok(Substitutions::empty()),
            (Var(var), b) => Self::bind(var, b),
            (a, Var(var)) => Self::bind(var, a),
            (Fn(i1, o1), Fn(i2, o2)) => Self::unify_vec(
                &[i1.as_ref().clone(), i2.as_ref().clone()],
                &[o1.as_ref().clone(), o2.as_ref().clone()],
            ),
            (Tuple(t1), Tuple(t2)) => Self::unify_vec(&t1.0, &t2.0),
            (Pointer(t1), Pointer(t2)) => t1.unify(t2),
            _ => Err(UnificationFail(lhs.clone(), rhs.clone())),
        }
    }
}

impl MonomorphicType {
    pub fn unify(&self, other: &MonomorphicType) -> Result<Substitutions, AlgorithmUError> {
        AlgorithmU::apply(self, other)
    }
}

impl Display for UnificationMismatch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot unify types: [{}] with [{}]; different structure",
            self.0.iter().join(", "),
            self.1.iter().join(", ")
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::HashMap;

    use nonempty_collections::nev;

    use crate::algorithm_u::AlgorithmUError;
    use crate::r#type::MonomorphicType::Constant;
    use crate::r#type::{fun, fun1, var, MonomorphicType, PrimitiveType, TVar, Tuple};
    use crate::substitution::Substitutions;
    use crate::traits::Substitutable;

    #[test]
    fn test_tautology_example_on_constants() {
        let a = Constant("A".to_string());
        let b = Constant("A".to_string());

        let s = a.unify(&b).unwrap();
        assert_eq!(s.into_inner(), HashMap::new());
    }

    #[test]
    fn test_different_constants_should_not_unify() {
        let a = Constant("A".to_string());
        let b = Constant("B".to_string());

        let e = a.unify(&b).unwrap_err();
        assert!(matches!(e, AlgorithmUError::UnificationFail(..)))
    }

    #[test]
    fn test_tautology_example_on_vars() {
        let a = var(0);
        let b = var(0);

        let s = a.unify(&b).unwrap();
        assert_eq!(s.into_inner(), HashMap::new());
    }

    #[test]
    fn test_variables_should_be_always_unified() {
        let a = TVar(1);
        let b = Constant("A".to_string());

        let s1 = MonomorphicType::Var(a).unify(&b).unwrap();
        let s2 = b.unify(&MonomorphicType::Var(a)).unwrap();

        assert_eq!(s1, s2);
        assert_eq!(s1, Substitutions::single(a, b))
    }

    #[test]
    fn test_aliasing() {
        let a = TVar(1);
        let b = TVar(2);

        let a_ = MonomorphicType::Var(a);
        let b_ = MonomorphicType::Var(b);

        let s1 = a_.unify(&b_).unwrap();
        let s2 = b_.unify(&a_).unwrap();

        assert_eq!(s1, Substitutions::single(a, b_.clone()));
        assert_eq!(s2, Substitutions::single(b, a_))
    }

    #[test]
    fn test_simple_function_unifying() {
        let a = fun(nev![var(1), Constant("A".to_string())], Tuple::unit());
        let b = fun(nev![var(1), var(2)], Tuple::unit());

        let s = a.unify(&b).unwrap();
        assert_eq!(s, Substitutions::single(TVar(2), Constant("A".to_string())));
    }

    #[test]
    fn test_aliasing_in_functions() {
        let a = fun(nev![var(1)], Tuple::unit());
        let b = fun(nev![var(2)], Tuple::unit());

        let s = a.unify(&b).unwrap();
        assert_eq!(s, Substitutions::single(TVar(1), var(2)));
    }

    #[test]
    fn test_functions_with_different_arity_should_not_unify() {
        let a = fun(nev![Constant("A".to_string())], Tuple::unit());
        let b = fun(
            nev![Constant("A".to_string()), Constant("B".to_string())],
            Tuple::unit(),
        );

        let s = a.unify(&b).unwrap_err();
        assert!(matches!(s, AlgorithmUError::UnificationFail(..)))
    }

    #[test]
    fn test_multiple_substitutions() {
        let a = fun(
            nev![fun1(var(1), PrimitiveType::Integral), var(1)],
            Tuple::unit(),
        );
        let b = fun(nev![var(2), Constant("A".to_string())], Tuple::unit());

        let s = a.unify(&b).unwrap();
        assert_eq!(
            s.into_inner(),
            HashMap::from([
                (TVar(1), Constant("A".to_string())),
                (
                    TVar(2),
                    fun1(Constant("A".to_string()), PrimitiveType::Integral)
                )
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
        let c = Constant("A".to_string());

        let s1 = a.unify(&b).unwrap();
        let s2 = b.unify(&a).unwrap();
        let s3 = c.unify(&b.substitute(&s2)).unwrap();
        let s4 = a.substitute(&s1).unify(&c).unwrap();

        assert_eq!(s1, Substitutions::single(TVar(1), b.clone()));
        assert_eq!(s2, Substitutions::single(TVar(2), a.clone()));
        assert_eq!(s3, Substitutions::single(TVar(1), c.clone()));
        assert_eq!(s4, Substitutions::single(TVar(2), c));
    }

    #[test]
    fn test_different_substitutions_of_same_variable() {
        let a = var(1);
        let b = Constant("A".to_string());
        let c = Constant("B".to_string());

        let s = a.unify(&b).unwrap();
        let e = a.substitute(&s).unify(&c).unwrap_err();

        assert_eq!(s, Substitutions::single(TVar(1), b));
        assert!(matches!(e, AlgorithmUError::UnificationFail(..)))
    }

    #[test]
    fn test_complex_unification() {
        let a = fun1(
            fun1(fun1(Constant("A".to_string()), var(1)), var(2)),
            var(3),
        );
        let b = fun1(
            var(3),
            fun1(var(2), fun1(var(1), Constant("A".to_string()))),
        );

        let s1 = a.unify(&b).unwrap();
        let s2 = b.unify(&a).unwrap();

        assert_eq!(s1, s2);
        assert_eq!(a.substitute(&s1), b.substitute(&s2));
    }
}

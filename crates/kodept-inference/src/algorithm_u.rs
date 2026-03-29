use crate::algorithm_u::AlgorithmUError::{InfiniteType, UnificationFail};
use crate::substitution::Substitutions;
use crate::traits::{FreeTypeVars, Substitutable};
use crate::r#type::{MonomorphicType, TVar};
use crate::utils::JoinedDisplay;
use MonomorphicType::*;
use derive_more::with_trait::{Display, Error, From};
use kodept_interning::{GlobalInterner, Interned};
use std::fmt::Formatter;

#[derive(Debug, Error, Clone)]
pub struct UnificationMismatch(
    pub Box<[Interned<MonomorphicType>]>,
    pub Box<[Interned<MonomorphicType>]>,
);

#[derive(Debug, Display, Error, From, Clone)]
pub enum AlgorithmUError {
    #[display("Cannot unify types: {_0} with {_1}")]
    #[from(ignore)]
    UnificationFail(Interned<MonomorphicType>, Interned<MonomorphicType>),
    #[display("Cannot construct an infinite type: {_0} ~ {_1}")]
    #[from(ignore)]
    InfiniteType(TVar, Interned<MonomorphicType>),
    UnificationMismatch(UnificationMismatch),
}

struct AlgorithmU;

impl AlgorithmU {
    fn occurs_check(var: &TVar, with: impl FreeTypeVars) -> bool {
        with.free_types().contains(var)
    }

    fn unify_vec(
        vec1: &[Interned<MonomorphicType>],
        vec2: &[Interned<MonomorphicType>],
    ) -> Result<Substitutions, AlgorithmUError> {
        match (vec1, vec2) {
            ([], []) => Ok(Substitutions::empty()),
            ([t1, ts1 @ ..], [t2, ts2 @ ..]) => {
                let s1 = t1.unify(t2)?;
                let s2 = Self::unify_vec(&ts1.substitute(&s1), &ts2.substitute(&s1))?;
                Ok(s1 + s2)
            }
            (t1, t2) => Err(UnificationMismatch(Box::from(t1), Box::from(t2)).into()),
        }
    }

    fn bind(var: &TVar, ty: &MonomorphicType) -> Result<Substitutions, AlgorithmUError> {
        match ty {
            Var(v) if var == v => Ok(Substitutions::empty()),
            _ if Self::occurs_check(var, ty) => Err(InfiniteType(*var, ty.intern())),
            _ => Ok(Substitutions::single(*var, &ty)),
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
            (Fn(i1, o1), Fn(i2, o2)) => Self::unify_vec(&[*i1, *o1], &[*i2, *o2]),
            (Tuple(t1), Tuple(t2)) => Self::unify_vec(&t1, &t2),
            (Pointer(t1), Pointer(t2)) => t1.unify(t2),
            (StaticArray(n1, t1), StaticArray(n2, t2)) if n1 == n2 => t1.unify(t2),
            _ => Err(UnificationFail(lhs.intern(), rhs.intern())),
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
            JoinedDisplay::enumerate(&self.0),
            JoinedDisplay::enumerate(&self.1)
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::algorithm_u::AlgorithmUError;
    use crate::substitution::Substitutions;
    use crate::traits::Substitutable;
    use crate::r#type::MonomorphicType::{Constant, Var};
    use crate::r#type::{MonomorphicType, PrimitiveType, TConstant, TVar};
    use kodept_interning::GlobalInterner;
    use std::collections::HashMap;

    #[test]
    fn test_tautology_example_on_constants() {
        let constant = TConstant::new();
        let a = Constant(constant);
        let b = Constant(constant);

        let s = a.unify(&b).unwrap();
        assert_eq!(s.into_inner(), HashMap::new());
    }

    #[test]
    fn test_different_constants_should_not_unify() {
        let a = MonomorphicType::constant();
        let b = MonomorphicType::constant();

        let e = a.unify(&b).unwrap_err();
        assert!(matches!(e, AlgorithmUError::UnificationFail(..)))
    }

    #[test]
    fn test_tautology_example_on_vars() {
        let var = TVar::new();
        let a = Var(var);
        let b = Var(var);

        let s = a.unify(&b).unwrap();
        assert_eq!(s.into_inner(), HashMap::new());
    }

    #[test]
    fn test_variables_should_be_always_unified() {
        let a = TVar::new();
        let b = MonomorphicType::constant();

        let s1 = Var(a).unify(&b).unwrap();
        let s2 = b.unify(&Var(a)).unwrap();

        assert_eq!(s1, s2);
        assert_eq!(s1, Substitutions::single(a, &b))
    }

    #[test]
    fn test_aliasing() {
        let a = TVar::new();
        let b = TVar::new();

        let a_ = Var(a);
        let b_ = Var(b);

        let s1 = a_.unify(&b_).unwrap();
        let s2 = b_.unify(&a_).unwrap();

        assert_eq!(s1, Substitutions::single(a, &b_));
        assert_eq!(s2, Substitutions::single(b, &a_))
    }

    #[test]
    fn test_simple_function_unifying() {
        let var1 = MonomorphicType::var();
        let var2 = TVar::new();
        let constant = MonomorphicType::constant();
        let a = MonomorphicType::fun(&var1, [&constant], MonomorphicType::UNIT);
        let b = MonomorphicType::fun(&var1, [&Var(var2)], MonomorphicType::UNIT);

        let s = a.unify(&b).unwrap();
        assert_eq!(s, Substitutions::single(var2, &constant));
    }

    #[test]
    fn test_aliasing_in_functions() {
        let [t1, t2] = TVar::new_many();
        let a = MonomorphicType::fun1(t1, MonomorphicType::UNIT);
        let b = MonomorphicType::fun1(t2, MonomorphicType::UNIT);

        let s = a.unify(&b).unwrap();
        assert_eq!(s, Substitutions::single(t1, &Var(t2)));
    }

    #[test]
    fn test_functions_with_different_arity_should_not_unify() {
        let constant = MonomorphicType::constant();

        let a = MonomorphicType::fun1(&constant, MonomorphicType::UNIT);
        let b = MonomorphicType::fun(
            &constant,
            [&MonomorphicType::constant()],
            MonomorphicType::UNIT,
        );

        let s = a.unify(&b).unwrap_err();
        assert!(matches!(s, AlgorithmUError::UnificationFail(..)))
    }

    #[test]
    fn test_multiple_substitutions() {
        let var1 = TVar::new();
        let var2 = TVar::new();
        let constant = MonomorphicType::constant();

        let a = MonomorphicType::fun(
            &MonomorphicType::fun1(&Var(var1), PrimitiveType::u8()),
            [&Var(var1)],
            MonomorphicType::UNIT,
        );
        let b = MonomorphicType::fun(&Var(var2), [&constant], MonomorphicType::UNIT);

        let s = a.unify(&b).unwrap();
        assert_eq!(
            s.into_inner(),
            HashMap::from([
                (var1, constant.intern()),
                (
                    var2,
                    MonomorphicType::fun1(&constant, PrimitiveType::u8()).intern()
                )
            ])
        )
    }

    #[test]
    fn test_infinite_substitution() {
        let var1 = MonomorphicType::var();
        let a = var1.clone();
        let b = MonomorphicType::fun1(&var1, &MonomorphicType::UNIT);

        let e = a.unify(&b).unwrap_err();
        assert!(matches!(e, AlgorithmUError::InfiniteType { .. }))
    }

    #[test]
    fn test_transitive_substitutions() {
        let var1 = TVar::new();
        let var2 = TVar::new();

        let a = Var(var1);
        let b = Var(var2);
        let c = MonomorphicType::constant();

        let s1 = a.unify(&b).unwrap();
        let s2 = b.unify(&a).unwrap();
        let s3 = c.unify(&b.substitute(&s2)).unwrap();
        let s4 = a.substitute(&s1).unify(&c).unwrap();

        assert_eq!(s1, Substitutions::single(var1, &b));
        assert_eq!(s2, Substitutions::single(var2, &a));
        assert_eq!(s3, Substitutions::single(var1, &c));
        assert_eq!(s4, Substitutions::single(var2, &c));
    }

    #[test]
    fn test_different_substitutions_of_same_variable() {
        let var1 = TVar::new();
        let a = Var(var1);
        let b = MonomorphicType::constant();
        let c = MonomorphicType::constant();

        let s = a.unify(&b).unwrap();
        let e = a.substitute(&s).unify(&c).unwrap_err();

        assert_eq!(s, Substitutions::single(var1, &b));
        assert!(matches!(e, AlgorithmUError::UnificationFail(..)))
    }

    #[test]
    fn test_complex_unification() {
        let var1 = TVar::new();
        let var2 = TVar::new();
        let var3 = TVar::new();
        let constant = MonomorphicType::constant();

        let a = MonomorphicType::fun1(
            &MonomorphicType::fun1(&MonomorphicType::fun1(&constant, &Var(var1)), &Var(var2)),
            &Var(var3),
        );
        let b = MonomorphicType::fun(&Var(var3), [&Var(var2), &Var(var1)], constant.clone());

        let s1 = a.unify(&b).unwrap();
        let s2 = b.unify(&a).unwrap();

        assert_eq!(s1, s2);
        assert_eq!(a.substitute(&s1), b.substitute(&s1));
        let h = MonomorphicType::fun1(&constant, &constant);
        assert_eq!(
            a.substitute(&s1),
            MonomorphicType::fun1(
                &MonomorphicType::fun1(&h, &h),
                &MonomorphicType::fun1(&h, &h)
            )
            .intern()
        )
    }
}

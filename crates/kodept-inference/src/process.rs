use crate::assumption::{AssumptionSet, TypeTable};
use crate::constraint::{
    Constraint, Constraints, ConstraintsSolverError, ExtendConstraints, explicit_cst,
};
use crate::substitution::Substitutions;
use crate::traits::Substitutable;
use crate::r#type::{MonomorphicType, PolymorphicType};
use derive_more::{Display, Error, From};
use kodept_interning::{InternInto, Interned};
use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Display, Error, From)]
pub enum InferError<Name, E> {
    #[from(ignore)]
    External(E),
    FailedConstraints(ConstraintsSolverError),
    #[from(ignore)]
    UnknownName(#[error(not(source))] Name),
}

#[derive(Debug)]
pub struct PartialInfer<Name> {
    pub assumptions: AssumptionSet<Name>,
    pub constraints: Constraints,
    pub current_type: Interned<MonomorphicType>,
}

impl<Name> PartialInfer<Name>
where
    Name: Hash + Eq,
{
    pub fn new(current_type: impl InternInto<MonomorphicType>) -> Self {
        Self {
            assumptions: AssumptionSet::empty(),
            constraints: Constraints::new(),
            current_type: current_type.intern_into(),
        }
    }

    pub(crate) fn default() -> Self {
        Self::new(MonomorphicType::UNIT)
    }

    pub fn with_assumption(mut self, key: Name, value: impl InternInto<MonomorphicType>) -> Self {
        self.assumptions
            .push(key, Cow::Borrowed(&[value.intern_into()]));
        self
    }

    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_constraints(mut self, iter: impl ExtendConstraints) -> Self {
        iter.append_into(&mut self.constraints);
        self
    }

    pub fn with_assumptions(mut self, set: AssumptionSet<Name>) -> Self {
        self.assumptions.merge(set);
        self
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint)
    }

    pub fn add_assumption(&mut self, key: Name, value: impl InternInto<MonomorphicType>) {
        self.assumptions
            .push(key, Cow::Borrowed(&[value.intern_into()]))
    }

    pub fn with_type(mut self, ty: impl InternInto<MonomorphicType>) -> Self {
        self.current_type = ty.intern_into();
        self
    }

    pub fn resolve<E>(
        mut self,
        mut external_symbols_callback: impl FnMut(&Name) -> Result<Option<PolymorphicType>, E>,
    ) -> Result<(Substitutions, Interned<MonomorphicType>), Vec<InferError<Name, E>>>
    where
        Name: Debug,
    {
        let mut errors = vec![];

        for (key, set) in self.assumptions.into_iter() {
            match external_symbols_callback(&key) {
                Ok(None) => errors.push(InferError::UnknownName(key)),
                Ok(Some(s)) => self
                    .constraints
                    .extend(set.into_iter().map(|it| explicit_cst(it.0, s.clone()))),
                Err(e) => errors.push(InferError::External(e)),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let substitutions = self
            .constraints
            .solve()
            .map_err(|it| vec![InferError::FailedConstraints(it)])?;
        let resulting_type = self.current_type.substitute(&substitutions);
        Ok((substitutions, resulting_type))
    }

    pub fn merge(&mut self, other: Self) -> Interned<MonomorphicType> {
        self.assumptions.merge(other.assumptions);
        self.constraints.merge(other.constraints);
        other.current_type
    }
}

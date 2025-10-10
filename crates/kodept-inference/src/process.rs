use crate::assumption::{AssumptionSet, TypeTable};
use crate::constraint::{Constraint, ConstraintsSolverError, explicit_cst};
use crate::substitution::Substitutions;
use crate::traits::Substitutable;
use crate::r#type::{MonomorphicType, PolymorphicType};
use derive_more::{Display, Error, From};
use kodept_interning::{InternInto, Interned};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::Hash;

const CONSTRAINTS_SIZE: usize = 4;

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
    pub constraints: SmallVec<[Constraint; CONSTRAINTS_SIZE]>,
    pub current_type: Interned<MonomorphicType>,
}

impl<Name> PartialInfer<Name>
where
    Name: Hash + Eq,
{
    pub fn new(current_type: impl InternInto<MonomorphicType>) -> Self {
        Self {
            assumptions: AssumptionSet::empty(),
            constraints: SmallVec::new(),
            current_type: current_type.intern_into(),
        }
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

    pub fn with_constraints(mut self, iter: impl IntoIterator<Item = Constraint>) -> Self {
        self.constraints.extend(iter);
        self
    }

    pub fn with_assumptions(mut self, set: AssumptionSet<Name>) -> Self {
        self.assumptions.merge(set);
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

        let substitutions = Constraint::solve(self.constraints.into_vec())
            .map_err(|it| vec![InferError::FailedConstraints(it)])?;
        let resulting_type = self.current_type.substitute(&substitutions);
        Ok((substitutions, resulting_type))
    }
}

pub trait Infer<Item, Name> {
    type Error;

    fn partial_infer<'a>(
        &'a mut self,
        value: &'a Item,
    ) -> impl Future<Output = Result<PartialInfer<Name>, Self::Error>>;
}

#[cfg(test)]
mod tests {
    use crate::assumption::TypeTable;
    use crate::constraint::{eq_cst, implicit_cst};
    use crate::process::{Infer, InferError, PartialInfer};
    use crate::r#type::{MonomorphicType, PrimitiveType, TVar};
    use kodept_interning::{GlobalInterner, Interned};
    use std::collections::HashSet;
    use std::convert::Infallible;
    use std::num::NonZeroU8;
    use std::ops::Deref;
    use std::pin::{Pin, pin};
    use std::task::{Context, Poll, Waker};

    type Name = &'static str;
    type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

    #[derive(Debug, Default)]
    struct AlgorithmW {
        monomorphic_set: HashSet<TVar>,
    }

    struct Var(Name);
    struct Lambda {
        bind: Var,
        expr: Language,
    }
    struct App {
        func: Language,
        arg: Language,
    }

    struct Let {
        binder: Language,
        bind: Var,
        usage: Language,
    }

    enum Language {
        Var(Var),
        Lambda(Box<Lambda>),
        App(Box<App>),
        Let(Box<Let>),
    }

    impl Infer<Var, Name> for AlgorithmW {
        type Error = Infallible;

        async fn partial_infer(&mut self, item: &Var) -> Result<PartialInfer<Name>, Self::Error> {
            let ty = TVar::new();
            Ok(PartialInfer::new(ty).with_assumption(item.0, ty))
        }
    }

    impl Infer<Lambda, Name> for AlgorithmW {
        type Error = Infallible;

        async fn partial_infer(
            &mut self,
            Lambda { bind, expr }: &Lambda,
        ) -> Result<PartialInfer<Name>, Self::Error> {
            let tv = TVar::new();
            self.monomorphic_set.insert(tv);
            let PartialInfer {
                assumptions: mut as1,
                constraints: cs1,
                current_type: t1,
            } = self.partial_infer(expr).await?;

            let tys = as1.resolve_take(bind.0);
            let eq_cs = tys.iter().cloned().map(|it| eq_cst(tv, it.0));

            Ok(PartialInfer::new(MonomorphicType::fun1(tv, t1.0))
                .with_constraints(eq_cs)
                .with_constraints(cs1)
                .with_assumptions(as1))
        }
    }

    impl Infer<App, Name> for AlgorithmW {
        type Error = Infallible;

        async fn partial_infer(
            &mut self,
            App { func, arg }: &App,
        ) -> Result<PartialInfer<Name>, Self::Error> {
            let PartialInfer {
                assumptions: as1,
                constraints: cs1,
                current_type: t1,
            } = self.partial_infer(arg).await?;
            let PartialInfer {
                assumptions: as2,
                constraints: cs2,
                current_type: t2,
            } = self.partial_infer(func).await?;
            let tv = TVar::new();

            Ok(PartialInfer::new(tv)
                .with_assumptions(as1)
                .with_assumptions(as2)
                .with_constraints(cs1)
                .with_constraints(cs2)
                .with_constraint(eq_cst(t2.0, MonomorphicType::fun1(t1.0, tv))))
        }
    }

    impl Infer<Let, Name> for AlgorithmW {
        type Error = Infallible;

        async fn partial_infer(
            &mut self,
            Let {
                binder,
                bind,
                usage,
            }: &Let,
        ) -> Result<PartialInfer<Name>, Self::Error> {
            let PartialInfer {
                assumptions: mut as1,
                constraints: cs1,
                current_type: t1,
            } = self.partial_infer(binder).await?;
            let PartialInfer {
                assumptions: as2,
                constraints: cs2,
                current_type: t2,
            } = self.partial_infer(usage).await?;

            as1.merge(as2);

            let tys = as1.resolve_take(bind.0);
            let im_cs = tys
                .iter()
                .cloned()
                .map(|it| implicit_cst(it.0, self.monomorphic_set.clone(), t1.0));

            Ok(PartialInfer::new(t2.0)
                .with_constraints(im_cs)
                .with_constraints(cs1)
                .with_constraints(cs2)
                .with_assumptions(as1))
        }
    }

    impl Infer<Language, Name> for AlgorithmW {
        type Error = Infallible;

        #[allow(refining_impl_trait)]
        fn partial_infer<'a>(
            &'a mut self,
            value: &'a Language,
        ) -> BoxedFuture<'a, Result<PartialInfer<Name>, Self::Error>> {
            match value {
                Language::Var(x) => Box::pin(self.partial_infer(x)),
                Language::Lambda(x) => Box::pin(self.partial_infer(x.deref())),
                Language::App(x) => Box::pin(self.partial_infer(x.deref())),
                Language::Let(x) => Box::pin(self.partial_infer(x.deref())),
            }
        }
    }

    // Look ma! We have haskell at home!
    fn var(name: &'static str) -> Language {
        Language::Var(Var(name))
    }

    fn let_(bind: &'static str, binder: Language, usage: Language) -> Language {
        Language::Let(Box::new(Let {
            binder,
            bind: Var(bind),
            usage,
        }))
    }

    fn app(func: Language, arg: Language) -> Language {
        Language::App(Box::new(App { func, arg }))
    }

    fn lambda(bind: &'static str, expr: Language) -> Language {
        Language::Lambda(Box::new(Lambda {
            bind: Var(bind),
            expr,
        }))
    }

    fn run_blocking<T>(fut: impl Future<Output = T>) -> T {
        let mut ctx = Context::from_waker(Waker::noop());
        let mut fut = pin!(fut);
        loop {
            match fut.as_mut().poll(&mut ctx) {
                Poll::Ready(x) => return x,
                Poll::Pending => continue,
            }
        }
    }

    macro_rules! assert_type_matches {
        ($a:expr, $b:pat $(if $cond:expr)?) => {
            match $a {
                $b $(if $cond)? => {},
                _ => {
                    dbg!($a);
                    assert!(false, "Expected a different type")
                }
            }
        };
    }

    #[test]
    fn test_identity_fn() {
        let mut solver = AlgorithmW::default();
        // \x. x
        let expr = lambda("x", var("x"));
        let partial = run_blocking(solver.partial_infer(&expr)).unwrap();
        let (_, t) = partial.resolve::<Infallible>(|_| Ok(None)).unwrap();

        assert_type_matches!(t.0, MonomorphicType::Fn(
            Interned(MonomorphicType::Var(TVar { index: a })),
            Interned(MonomorphicType::Var(TVar { index: b })),
        ) if a == b);
    }

    #[test]
    fn test_external_names() {
        const I32: PrimitiveType = PrimitiveType::custom(true, NonZeroU8::new(32).unwrap());

        // \n. let x = succ(n) in succ(x)
        let expr = lambda(
            "n",
            let_("x", app(var("succ"), var("n")), app(var("succ"), var("x"))),
        );
        let mut solver = AlgorithmW::default();
        let partial = run_blocking(solver.partial_infer(&expr)).unwrap();
        let (_, t) = partial
            .resolve::<Infallible>(|name| {
                if name == &"succ" {
                    Ok(Some(
                        MonomorphicType::fun1(
                            MonomorphicType::primitive(I32),
                            MonomorphicType::primitive(I32),
                        )
                        .generalize(&HashSet::new()),
                    ))
                } else {
                    Ok(None)
                }
            })
            .unwrap();

        const THIRTY_TWO: NonZeroU8 = NonZeroU8::new(32).unwrap();
        assert_type_matches!(
            t.0,
            MonomorphicType::Fn(
                Interned(MonomorphicType::Primitive(Interned(PrimitiveType::I(
                    THIRTY_TWO
                )))),
                Interned(MonomorphicType::Primitive(Interned(PrimitiveType::I(
                    THIRTY_TWO
                ))))
            )
        );
    }

    #[test]
    fn test_tuples() {
        // \z. let x = dup(z) in (\y. dup(y))(x)
        let expr = lambda(
            "z",
            let_(
                "x",
                app(var("dup"), var("z")),
                app(lambda("y", app(var("dup"), var("x"))), var("x")),
            ),
        );

        let mut solver = AlgorithmW::default();
        let partial = run_blocking(solver.partial_infer(&expr)).unwrap();

        let t = TVar::new();
        let dup_t =
            MonomorphicType::fun1(t, MonomorphicType::tuple([t, t])).generalize(&HashSet::new());
        let (_, t_result) = partial
            .resolve::<Infallible>(|name| {
                if name != &"dup" {
                    return Ok(None);
                }
                Ok(Some(dup_t.clone()))
            })
            .unwrap();

        assert_type_matches!(
            t_result.0,
            MonomorphicType::Fn(
                Interned(MonomorphicType::Var(a)),
                Interned(MonomorphicType::Tuple(ts))
            ) if ts.as_ref() == [MonomorphicType::tuple([*a, *a]).intern(), MonomorphicType::tuple([*a, *a]).intern()]
        );
    }

    #[test]
    fn test_church_encoding() {
        //zero = \f. \x. x                   :: a -> b -> b
        //one  = \f. \x. f x                 :: (a -> b) -> a -> b
        //plus = \m. \n. \f. \x. m f (n f x) :: (a -> b -> c) -> (a -> d -> b) -> a -> d -> c

        let zero = lambda("f", lambda("x", var("x")));
        let one = lambda("f", lambda("x", app(var("f"), var("x"))));
        let plus = lambda(
            "m",
            lambda(
                "n",
                lambda(
                    "f",
                    lambda(
                        "x",
                        app(
                            app(var("m"), var("f")),
                            app(app(var("n"), var("f")), var("x")),
                        ),
                    ),
                ),
            ),
        );

        fn solve(
            expr: &Language,
        ) -> Result<&'static MonomorphicType, Vec<InferError<Name, Infallible>>> {
            let mut solver = AlgorithmW::default();
            let partial = run_blocking(solver.partial_infer(expr)).unwrap();
            partial.resolve::<Infallible>(|_| Ok(None)).map(|it| it.1.0)
        }

        let zt = solve(&zero).unwrap().generalize(&HashSet::new());
        let ot = solve(&one).unwrap().generalize(&HashSet::new());
        let pt = solve(&plus).unwrap().generalize(&HashSet::new());

        println!("{zt}\n{ot}\n{pt}");
    }
}

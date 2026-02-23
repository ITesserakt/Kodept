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

#[derive(Debug, Clone)]
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

        let substitutions = Constraint::solve(self.constraints.into_vec())
            .map_err(|it| vec![InferError::FailedConstraints(it)])?;
        let resulting_type = self.current_type.substitute(&substitutions);
        Ok((substitutions, resulting_type))
    }
}

pub type InferResult<Name, Error> = Result<PartialInfer<Name>, Error>;

pub trait InferFuture<Name, Error>: Future<Output = InferResult<Name, Error>> + Send {}

impl<Name, Error, T: Future<Output = InferResult<Name, Error>> + Send> InferFuture<Name, Error>
    for T
{
}

pub trait Infer<Item, Name> {
    type Error;

    fn partial_infer(value: Item) -> impl InferFuture<Name, Self::Error>;
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
    type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send + Sync>>;

    struct AlgorithmW;

    struct InLambda<T> {
        binds: HashSet<TVar>,
        value: Box<T>,
    }

    struct Var(Name);
    struct Lambda {
        bind: Var,
        bind_ty_stub: TVar,
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
        Lambda(InLambda<Lambda>),
        App(InLambda<App>),
        Let(InLambda<Let>),
    }

    impl<'a> Infer<&'a Var, Name> for AlgorithmW {
        type Error = Infallible;

        async fn partial_infer(item: &'a Var) -> Result<PartialInfer<Name>, Self::Error> {
            let ty = TVar::new();
            Ok(PartialInfer::new(ty).with_assumption(item.0, ty))
        }
    }

    impl<'a> Infer<&'a Lambda, Name> for AlgorithmW {
        type Error = Infallible;

        fn partial_infer(
            Lambda {
                bind,
                bind_ty_stub: tv,
                expr,
            }: &'a Lambda,
        ) -> impl Future<Output = Result<PartialInfer<Name>, Self::Error>> {
            async move {
                let PartialInfer {
                    assumptions: mut as1,
                    constraints: cs1,
                    current_type: t1,
                } = Self::partial_infer(expr).await?;

                let tys = as1.resolve_take(bind.0);
                let eq_cs = tys.iter().cloned().map(|it| eq_cst(*tv, it.0));

                Ok(PartialInfer::new(MonomorphicType::fun1(*tv, t1.0))
                    .with_constraints(eq_cs)
                    .with_constraints(cs1)
                    .with_assumptions(as1))
            }
        }
    }

    impl<'a> Infer<&'a App, Name> for AlgorithmW {
        type Error = Infallible;

        fn partial_infer(
            App { func, arg }: &'a App,
        ) -> impl Future<Output = Result<PartialInfer<Name>, Self::Error>> {
            async move {
                let PartialInfer {
                    assumptions: as1,
                    constraints: cs1,
                    current_type: t1,
                } = Self::partial_infer(arg).await?;
                let PartialInfer {
                    assumptions: as2,
                    constraints: cs2,
                    current_type: t2,
                } = Self::partial_infer(func).await?;
                let tv = TVar::new();

                Ok(PartialInfer::new(tv)
                    .with_assumptions(as1)
                    .with_assumptions(as2)
                    .with_constraints(cs1)
                    .with_constraints(cs2)
                    .with_constraint(eq_cst(t2.0, MonomorphicType::fun1(t1.0, tv))))
            }
        }
    }

    impl<'a> Infer<&'a InLambda<Let>, Name> for AlgorithmW {
        type Error = Infallible;

        fn partial_infer(
            InLambda { binds, value }: &'a InLambda<Let>,
        ) -> impl Future<Output = Result<PartialInfer<Name>, Self::Error>> {
            async move {
                let PartialInfer {
                    assumptions: mut as1,
                    constraints: cs1,
                    current_type: t1,
                } = Self::partial_infer(&value.binder).await?;
                let PartialInfer {
                    assumptions: as2,
                    constraints: cs2,
                    current_type: t2,
                } = Self::partial_infer(&value.usage).await?;

                as1.merge(as2);

                let tys = as1.resolve_take(value.bind.0);
                let im_cs = tys
                    .iter()
                    .cloned()
                    .map(|it| implicit_cst(it.0, binds.clone(), t1.0));

                Ok(PartialInfer::new(t2.0)
                    .with_constraints(im_cs)
                    .with_constraints(cs1)
                    .with_constraints(cs2)
                    .with_assumptions(as1))
            }
        }
    }

    impl<'a> Infer<&'a Language, Name> for AlgorithmW {
        type Error = Infallible;

        #[allow(refining_impl_trait)]
        fn partial_infer(
            value: &'a Language,
        ) -> BoxedFuture<'a, Result<PartialInfer<Name>, Self::Error>> {
            match value {
                Language::Var(x) => Box::pin(Self::partial_infer(x)),
                Language::Lambda(x) => Box::pin(Self::partial_infer(x.value.deref())),
                Language::App(x) => Box::pin(Self::partial_infer(x.value.deref())),
                Language::Let(x) => Box::pin(Self::partial_infer(x)),
            }
        }
    }

    // Look ma! We have haskell at home!
    fn var(name: &'static str) -> Language {
        Language::Var(Var(name))
    }

    fn let_(bind: &'static str, binder: Language, usage: Language) -> Language {
        Language::Let(InLambda {
            binds: HashSet::new(),
            value: Box::new(Let {
                binder,
                bind: Var(bind),
                usage,
            }),
        })
    }

    fn app(func: Language, arg: Language) -> Language {
        Language::App(InLambda {
            binds: HashSet::new(),
            value: Box::new(App { arg, func }),
        })
    }

    fn lambda(bind: &'static str, expr: Language) -> Language {
        Language::Lambda(InLambda {
            binds: Default::default(),
            value: Box::new(Lambda {
                bind: Var(bind),
                bind_ty_stub: TVar::new(),
                expr,
            }),
        })
    }

    // populate `InLambda::set` members
    fn correct_expr(mut expr: Language) -> Language {
        // propagate `bind_ty_stub` from lambdas to children
        fn step(expr: &mut Language, previous: &HashSet<TVar>) {
            match expr {
                Language::Var(_) => {}
                Language::Lambda(x) => {
                    x.binds.extend(previous);
                    x.binds.insert(x.value.bind_ty_stub);
                    step(&mut x.value.expr, &x.binds);
                }
                Language::App(x) => {
                    x.binds.extend(previous);
                    step(&mut x.value.func, &x.binds);
                    step(&mut x.value.arg, &x.binds);
                }
                Language::Let(x) => {
                    x.binds.extend(previous);
                    step(&mut x.value.usage, &x.binds);
                    step(&mut x.value.binder, &x.binds);
                }
            }
        }
        step(&mut expr, &HashSet::new());
        expr
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
                    assert!(false, "Expected a different type, got: {}", $a)
                }
            }
        };
    }

    #[test]
    fn test_identity_fn() {
        // \x. x
        let expr = correct_expr(lambda("x", var("x")));
        let partial = run_blocking(AlgorithmW::partial_infer(&expr)).unwrap();
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
        let expr = correct_expr(lambda(
            "n",
            let_("x", app(var("succ"), var("n")), app(var("succ"), var("x"))),
        ));
        let partial = run_blocking(AlgorithmW::partial_infer(&expr)).unwrap();
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
        let expr = correct_expr(lambda(
            "z",
            let_(
                "x",
                app(var("dup"), var("z")),
                app(lambda("y", app(var("dup"), var("x"))), var("x")),
            ),
        ));

        let partial = run_blocking(AlgorithmW::partial_infer(&expr)).unwrap();

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

        let zero = correct_expr(lambda("f", lambda("x", var("x"))));
        let one = correct_expr(lambda("f", lambda("x", app(var("f"), var("x")))));
        let plus = correct_expr(lambda(
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
        ));

        fn solve(
            expr: &Language,
        ) -> Result<&'static MonomorphicType, Vec<InferError<Name, Infallible>>> {
            let partial = run_blocking(AlgorithmW::partial_infer(expr)).unwrap();
            partial.resolve::<Infallible>(|_| Ok(None)).map(|it| it.1.0)
        }

        let zt = solve(&zero).unwrap().generalize(&HashSet::new());
        let ot = solve(&one).unwrap().generalize(&HashSet::new());
        let pt = solve(&plus).unwrap().generalize(&HashSet::new());

        println!("{zt}\n{ot}\n{pt}");
    }

    #[test]
    fn test_let_specialization() {
        // let id = \y. y in tuple (id one) (id true)
        let expr = correct_expr(let_(
            "id",
            lambda("y", var("y")),
            app(
                app(var("tuple"), app(var("id"), var("one"))),
                app(var("id"), var("true")),
            ),
        ));
        let partial = run_blocking(AlgorithmW::partial_infer(&expr)).unwrap();

        let [t1, t2] = TVar::new_many();
        let tuple_t = MonomorphicType::fun(t1, [t2], MonomorphicType::tuple([t1, t2]))
            .generalize(&HashSet::new());
        let true_t = MonomorphicType::primitive(PrimitiveType::Boolean).generalize(&HashSet::new());
        let one_t = MonomorphicType::primitive(PrimitiveType::i8()).generalize(&HashSet::new());

        let (_, t) = partial
            .resolve::<Infallible>(|&name| match name {
                "tuple" => Ok(Some(tuple_t.clone())),
                "true" => Ok(Some(true_t.clone())),
                "one" => Ok(Some(one_t.clone())),
                _ => Ok(None),
            })
            .unwrap();

        assert_type_matches!(t.0, MonomorphicType::Tuple(ts)
        if ts.as_ref() == [
            MonomorphicType::primitive(PrimitiveType::i8()).intern(),
            MonomorphicType::primitive(PrimitiveType::Boolean).intern()
        ]);
    }

    #[test]
    fn test_recursion() {
        // let length = \xs. succ (length (tail xs)) in length
        let expr = correct_expr(let_(
            "length",
            lambda(
                "xs",
                app(var("succ"), app(var("length"), app(var("tail"), var("xs")))),
            ),
            var("length"),
        ));
        let partial = run_blocking(AlgorithmW::partial_infer(&expr)).unwrap();

        let [t1] = TVar::new_many();
        let tail_t =
            MonomorphicType::fun1(MonomorphicType::pointer(t1), MonomorphicType::pointer(t1))
                .generalize(&HashSet::new());
        let succ_t = MonomorphicType::fun1(PrimitiveType::u8(), PrimitiveType::u8())
            .generalize(&HashSet::new());

        let (_, t) = partial
            .resolve::<Infallible>(|&name| match name {
                "tail" => Ok(Some(tail_t.clone())),
                "succ" => Ok(Some(succ_t.clone())),
                _ => Ok(None),
            })
            .unwrap();

        const EIGHT: NonZeroU8 = NonZeroU8::new(8).unwrap();
        assert_type_matches!(
            t.0,
            MonomorphicType::Fn(
                Interned(MonomorphicType::Pointer(Interned(MonomorphicType::Var(_)))),
                Interned(MonomorphicType::Primitive(Interned(PrimitiveType::U(
                    EIGHT
                ))))
            )
        );
    }

    #[test]
    fn test_mutual_recursion() {
        // let is_odd = \x. not (is_even (pred x))
        // in
        //   let is_even = \y. not (is_odd (pred y))
        //   in
        //     tuple is_odd is_even

        let expr = correct_expr(let_(
            "is_odd",
            lambda(
                "x",
                app(var("not"), app(var("is_even"), app(var("pred"), var("x")))),
            ),
            let_(
                "is_even",
                lambda(
                    "y",
                    app(var("not"), app(var("is_odd"), app(var("pred"), var("y")))),
                ),
                app(app(var("tuple"), var("is_odd")), var("is_even")),
            ),
        ));
        let partial = run_blocking(AlgorithmW::partial_infer(&expr)).unwrap();

        let not_t = MonomorphicType::fun1(PrimitiveType::Boolean, PrimitiveType::Boolean)
            .generalize(&HashSet::new());
        let [t1, t2] = TVar::new_many();
        let tuple_t = MonomorphicType::fun(t1, [t2], MonomorphicType::tuple([t1, t2]))
            .generalize(&HashSet::new());
        let pred_t = MonomorphicType::fun1(PrimitiveType::u8(), PrimitiveType::u8())
            .generalize(&HashSet::new());

        let errors = partial
            .resolve::<Infallible>(|&name| match name {
                "not" => Ok(Some(not_t.clone())),
                "tuple" => Ok(Some(tuple_t.clone())),
                "pred" => Ok(Some(pred_t.clone())),
                _ => Ok(None),
            })
            .unwrap_err();

        // despite being literally in `expr`, `is_even` is not properly defined still
        assert!(matches!(
            errors.as_slice(),
            [InferError::UnknownName("is_even")]
        ))
    }
}

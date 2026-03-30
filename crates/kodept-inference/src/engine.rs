use crate::constraint::{Constraint, eq_cst};
use crate::process::PartialInfer;
use crate::r#type::{MonomorphicType, PolymorphicType, PrimitiveType, TVar};
use kodept_interning::InternInto;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Debug)]
pub struct Branch<Name> {
    pub condition: PartialInfer<Name>,
    pub body: PartialInfer<Name>,
}

pub struct Parameter<'a, Name> {
    pub tv: TVar,
    pub bound: Annotation<'a, Name>,
    pub name: Name,
}

pub enum Annotation<'a, Name> {
    None,
    Single(Name),
    Tuple(Box<dyn Iterator<Item = Annotation<'a, Name>> + 'a>),
}

pub enum LangItem<'a, Name> {
    /// A constant referencing some value
    Value(Name),
    /// A type itself
    Type(PolymorphicType),
    /// A tuple of some types
    Tuple(Box<dyn Iterator<Item = PartialInfer<Name>> + 'a>),
    /// A set of branches
    Branches(Box<dyn Iterator<Item = Branch<Name>> + 'a>),
    Call {
        callable: PartialInfer<Name>,
        arguments: Box<dyn DoubleEndedIterator<Item = PartialInfer<Name>> + 'a>,
    },
    /// Pass through
    Forward(PartialInfer<Name>),
    Lambda {
        parameters: Box<dyn DoubleEndedIterator<Item = Parameter<'a, Name>> + 'a>,
        body: PartialInfer<Name>,
    },
    VariableRec {
        monomorphic_context: HashSet<TVar>,
        body: PartialInfer<Name>,
        bound: Annotation<'a, Name>,
        name: &'a Name,
    },
    Variable {
        body: PartialInfer<Name>,
        bound: Annotation<'a, Name>,
    },
}

#[derive(Debug, Default)]
pub struct DefaultTypeckContext;

impl<Name> Annotation<'_, Name> {
    fn apply_to_container(
        self,
        associate: impl InternInto<MonomorphicType>,
        context: &impl TypeckContext<Name>,
        partial: &mut PartialInfer<Name>,
    ) where
        Name: Hash + Eq,
    {
        match self {
            Annotation::None => {}
            Annotation::Single(x) => partial.assumptions.push_single(x, associate),
            Annotation::Tuple(items) => {
                let tuple_ty = items.into_iter().map(|it| {
                    let tv = context.allocate_type_var();
                    it.apply_to_container(tv, context, partial);
                    tv
                });
                let tuple_ty = MonomorphicType::tuple(tuple_ty);
                partial.constraints.push(eq_cst(tuple_ty, associate));
            }
        }
    }
}

pub trait TypeckContext<Name>: Sized {
    fn allocate_type_var(&self) -> TVar {
        TVar::new()
    }

    fn eval(&self, item: LangItem<Name>) -> PartialInfer<Name>
    where
        Name: Hash + Eq,
    {
        match item {
            LangItem::Value(name) => {
                let tv = self.allocate_type_var();
                PartialInfer::new(tv).with_assumption(name, tv)
            }
            LangItem::Type(ty) => {
                let s = ty
                    .bindings
                    .iter()
                    .map(|it| (*it, MonomorphicType::Var(self.allocate_type_var())))
                    .collect();
                PartialInfer::new((ty.binding_type & &s).0)
            }
            LangItem::Tuple(items) => {
                let mut container = PartialInfer::default();
                let tuple_ty = MonomorphicType::tuple(items.map(|it| container.merge(it).0));
                container.with_type(tuple_ty)
            }
            LangItem::Branches(mut branches) => {
                let Some(Branch { condition, body }) = branches.next() else {
                    return PartialInfer::default();
                };
                let mut container = PartialInfer::new(body.current_type.0)
                    .with_constraint(eq_cst(condition.current_type.0, PrimitiveType::Boolean))
                    .with_assumptions(condition.assumptions)
                    .with_constraints(condition.constraints)
                    .with_assumptions(body.assumptions)
                    .with_constraints(body.constraints);

                for branch in branches {
                    let condition_ty = container.merge(branch.condition);
                    let body_ty = container.merge(branch.body);
                    container.constraints.extend([
                        eq_cst(condition_ty.0, PrimitiveType::Boolean),
                        eq_cst(body_ty.0, container.current_type.0),
                    ]);
                }
                container
            }
            LangItem::Call {
                callable,
                mut arguments,
            } => {
                let tv = self.allocate_type_var();
                let mut container = PartialInfer::new(tv)
                    .with_assumptions(callable.assumptions)
                    .with_constraints(callable.constraints);
                let expected_callable_type = match arguments.next() {
                    None => MonomorphicType::fun1(MonomorphicType::UNIT, tv),
                    Some(arg1) => {
                        let arg1_ty = container.merge(arg1);
                        MonomorphicType::fun(
                            arg1_ty.0,
                            arguments.map(|it| container.merge(it).0),
                            MonomorphicType::Var(tv),
                        )
                    }
                };
                container.with_constraint(eq_cst(callable.current_type.0, expected_callable_type))
            }
            LangItem::Forward(x) => x,
            LangItem::Lambda {
                mut parameters,
                body,
            } => {
                let apply_parameter = |parameter, container: &mut PartialInfer<Name>| {
                    let Parameter { tv, bound, name } = parameter;
                    let parameter_assumptions = container.assumptions.resolve_take(&name);
                    container
                        .constraints
                        .extend(parameter_assumptions.into_iter().map(|it| eq_cst(it.0, tv)));
                    bound.apply_to_container(tv, self, container);
                    tv
                };

                let mut container = PartialInfer::default();
                let body_ty = container.merge(body);
                let expected_result_ty = match parameters.next() {
                    None => MonomorphicType::fun1(MonomorphicType::UNIT, body_ty.0),
                    Some(param1) => {
                        let param1_ty = apply_parameter(param1, &mut container);
                        MonomorphicType::fun(
                            param1_ty,
                            parameters.map(|it| apply_parameter(it, &mut container)),
                            body_ty.0.clone(),
                        )
                    }
                };
                container.with_type(expected_result_ty)
            }
            LangItem::VariableRec {
                monomorphic_context,
                body,
                bound,
                name,
            } => {
                let mut container = PartialInfer::default();
                let body_ty = container.merge(body);
                let monomorphic_context = Arc::new(monomorphic_context);
                let variable_assumptions = container.assumptions.resolve_take(name);

                container
                    .constraints
                    .extend(variable_assumptions.into_iter().map(|it| {
                        Constraint::ImplicitInstance {
                            t1: it,
                            ctx: monomorphic_context.clone(),
                            t2: body_ty,
                        }
                    }));
                bound.apply_to_container(body_ty.0, self, &mut container);

                container.with_type(body_ty.0)
            }
            LangItem::Variable { body, bound } => {
                let mut container = body;
                bound.apply_to_container(container.current_type.0, self, &mut container);

                container
            }
        }
    }
}

impl<Name> TypeckContext<Name> for DefaultTypeckContext {}

#[cfg(test)]
mod tests {
    use crate::constraint::{eq_cst, implicit_cst};
    use crate::engine::{
        Annotation, Branch, DefaultTypeckContext, LangItem, Parameter, TypeckContext,
    };
    use crate::process::{InferError, PartialInfer};
    use crate::r#type::{MonomorphicType, PolymorphicType, PrimitiveType, TVar};
    use kodept_interning::Interned;
    use std::collections::HashSet;
    use std::convert::Infallible;
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    enum Expr {
        Value(&'static str),
        IntLit,
        BoolLit,
        Foreign(PolymorphicType),
        Tuple(Vec<Expr>),
        IfElse {
            condition: Box<Expr>,
            body: Box<Expr>,
            otherwise: Box<Expr>,
        },
        Block(Vec<Expr>),
        Call {
            callable: Box<Expr>,
            params: Vec<Expr>,
        },
        Return(Box<Expr>),
        Lambda {
            params: Vec<&'static str>,
            body: Box<Expr>,
        },
        Variable {
            name: &'static str,
            body: Box<Expr>,
            recursive: bool,
        },
    }

    // Look ma! Finished Kodept!

    fn v(referrer: &'static str) -> Expr {
        Expr::Value(referrer)
    }

    fn tuple(items: impl IntoIterator<Item = Expr>) -> Expr {
        Expr::Tuple(Vec::from_iter(items))
    }

    fn int() -> Expr {
        Expr::IntLit
    }

    fn bool() -> Expr {
        Expr::BoolLit
    }

    fn if_then_else(condition: Expr, then: Expr, r#else: Expr) -> Expr {
        Expr::IfElse {
            condition: Box::new(condition),
            body: Box::new(then),
            otherwise: Box::new(r#else),
        }
    }

    struct Scope<'a>(&'a mut Vec<Expr>);

    impl<'a> Scope<'a> {
        fn add(&mut self, value: Expr) {
            self.0.push(value);
        }

        fn ret(&mut self, value: Expr) {
            self.0.push(Expr::Return(Box::new(value)));
        }

        fn var(&mut self, name: &'static str, body: Expr) {
            self.0.push(Expr::Variable {
                name,
                body: Box::new(body),
                recursive: false,
            })
        }

        fn rec(&mut self, name: &'static str, body: Expr) {
            self.0.push(Expr::Variable {
                name,
                body: Box::new(body),
                recursive: true,
            })
        }
    }

    fn scope<T>(block: impl FnOnce(Scope) -> T) -> Expr {
        let mut items = Vec::new();
        block(Scope(&mut items));
        Expr::Block(items)
    }

    fn foreign(t: impl Into<PolymorphicType>) -> Expr {
        Expr::Foreign(t.into())
    }

    trait Arg {
        fn into_expr(self) -> Expr;
    }

    impl Arg for &'static str {
        fn into_expr(self) -> Expr {
            Expr::Value(self)
        }
    }

    impl Arg for Expr {
        fn into_expr(self) -> Expr {
            self
        }
    }

    impl Expr {
        fn call(&self, args: impl IntoIterator<Item: Arg>) -> Self {
            Self::Call {
                callable: Box::new(self.clone()),
                params: args.into_iter().map(|it| it.into_expr()).collect(),
            }
        }

        fn typeck_block<'a>(
            exprs: &'a [Expr],
            lambda_params: &mut HashSet<TVar>,
            context: &impl TypeckContext<&'static str>,
        ) -> LangItem<'a, &'static str> {
            let mut container = PartialInfer::new(context.allocate_type_var());
            let mut any_returns = false;
            let mut declared_variables = Vec::new();
            let ctx = Arc::new(lambda_params.clone());
            fn resolve_variables(
                item_partial: &mut PartialInfer<&str>,
                variables: &[(&'static str, Interned<MonomorphicType>)],
                ctx: Arc<HashSet<TVar>>,
            ) {
                for (var_name, var_type) in variables {
                    let assumptions = item_partial.assumptions.resolve_take(var_name);
                    item_partial.constraints.extend(
                        assumptions
                            .into_iter()
                            .map(|it| implicit_cst(it.0, &ctx, var_type.0)),
                    );
                }
            }

            for item in exprs {
                let mut item_partial = item.typeck_step(lambda_params, context);
                resolve_variables(&mut item_partial, &declared_variables, ctx.clone());
                let item_ty = container.merge(item_partial);
                match item {
                    Expr::Return(_) => {
                        any_returns = true;
                        container
                            .constraints
                            .push(eq_cst(item_ty.0, container.current_type.0));
                    }
                    Expr::Variable { name, .. } => declared_variables.push((*name, item_ty)),
                    _ => {}
                };
            }
            if !any_returns {
                container
                    .constraints
                    .push(eq_cst(MonomorphicType::UNIT, container.current_type.0));
            }
            LangItem::Forward(container)
        }

        fn typeck_step(
            &self,
            lambda_params: &mut HashSet<TVar>,
            context: &impl TypeckContext<&'static str>,
        ) -> PartialInfer<&'static str> {
            let item = match self {
                Expr::Value(x) => LangItem::Value(*x),
                Expr::IntLit => LangItem::Type(PrimitiveType::i8().into()),
                Expr::BoolLit => LangItem::Type(PrimitiveType::bool().into()),
                Expr::Foreign(x) => LangItem::Type(x.clone()),
                Expr::Tuple(items) => LangItem::Tuple(Box::new(
                    items
                        .iter()
                        .map(|it| it.typeck_step(lambda_params, context)),
                )),
                Expr::IfElse {
                    condition,
                    body,
                    otherwise,
                } => LangItem::Branches(Box::new(
                    [
                        Branch {
                            condition: condition.typeck_step(lambda_params, context),
                            body: body.typeck_step(lambda_params, context),
                        },
                        Branch {
                            condition: bool().typeck_step(lambda_params, context),
                            body: otherwise.typeck_step(lambda_params, context),
                        },
                    ]
                    .into_iter(),
                )),
                Expr::Block(items) => Self::typeck_block(items, lambda_params, context),
                Expr::Call { callable, params } => LangItem::Call {
                    callable: callable.typeck_step(lambda_params, context),
                    arguments: Box::new(
                        params
                            .iter()
                            .map(|it| it.typeck_step(lambda_params, context)),
                    ),
                },
                Expr::Return(x) => LangItem::Forward(x.typeck_step(lambda_params, context)),
                Expr::Lambda { params, body } => {
                    let param_tvs = params
                        .iter()
                        .map(|_| context.allocate_type_var())
                        .collect::<Vec<_>>();
                    lambda_params.extend(param_tvs.iter().cloned());
                    let result = LangItem::Lambda {
                        parameters: Box::new(params.iter().zip(param_tvs.clone()).map(|it| {
                            Parameter {
                                bound: Annotation::None,
                                name: *it.0,
                                tv: it.1,
                            }
                        })),
                        body: body.typeck_step(lambda_params, context),
                    };
                    param_tvs.into_iter().for_each(|it| {
                        lambda_params.remove(&it);
                    });
                    result
                }
                Expr::Variable {
                    body,
                    recursive: false,
                    ..
                } => LangItem::Variable {
                    bound: Annotation::None,
                    body: body.typeck_step(lambda_params, context),
                },
                Expr::Variable {
                    body,
                    name,
                    recursive: true,
                } => LangItem::VariableRec {
                    monomorphic_context: lambda_params.clone(),
                    body: body.typeck_step(lambda_params, context),
                    bound: Annotation::None,
                    name,
                },
            };
            context.eval(item)
        }

        fn typeck(&self, context: &impl TypeckContext<&'static str>) -> PartialInfer<&'static str> {
            let mut set = HashSet::new();
            self.typeck_step(&mut set, context)
        }

        fn typeck_full(
            &self,
            context: &impl TypeckContext<&'static str>,
        ) -> Result<PolymorphicType, Vec<InferError<&'static str, Infallible>>> {
            let partial = self.typeck(context);
            match partial.resolve(|_| Ok::<_, Infallible>(None)) {
                Ok((_, t)) => Ok(t.generalize(&HashSet::new())),
                Err(e) => Err(e),
            }
        }
    }

    fn lambda(params: impl IntoIterator<Item = &'static str>, body: Expr) -> Expr {
        Expr::Lambda {
            params: Vec::from_iter(params),
            body: Box::new(body),
        }
    }

    fn test(input: Expr, output: &'static str) {
        let ctx = DefaultTypeckContext;
        match input.typeck_full(&ctx) {
            Ok(x) => assert_eq!(format!("{x}"), output, "Types are not equal: {x}"),
            Err(e) => assert!(false, "Cannot typeck: {e:?}"),
        }
    }

    #[test]
    fn test_identity_fn() {
        test(
            lambda(["x"], scope(|mut it| it.ret(v("x")))),
            "∀a => a -> a",
        );
    }

    #[test]
    fn test_tuples() {
        test(
            lambda(
                ["z"],
                scope(|mut it| {
                    let dup = lambda(["y"], tuple([v("y"), v("y")]));
                    it.var("x", dup.call([v("z")]));
                    it.ret(dup.call([v("x")]))
                }),
            ),
            "∀a => a -> ((a, a), (a, a))",
        )
    }

    #[test]
    fn test_peano_numbers() {
        test(
            scope(|mut it| {
                it.var("zero", lambda(["f", "x"], v("x")));
                it.var("one", lambda(["f", "x"], v("f").call(["x"])));
                it.var(
                    "plus",
                    lambda(
                        ["m", "n", "f", "x"],
                        v("m").call([v("f"), v("n").call(["f", "x"])]),
                    ),
                );
                it.var(
                    "plus3",
                    lambda(
                        ["a", "b", "c"],
                        v("plus").call([v("plus").call(["a", "b"]), v("c")]),
                    ),
                );
                it.ret(v("plus3").call(["one", "zero", "one"]));
            }),
            "∀a => (a -> a) -> a -> a",
        )
    }

    #[test]
    #[should_panic(expected = r#"Cannot typeck: [UnknownName("identity")]"#)]
    fn test_use_before_declaration() {
        test(
            scope(|mut it| {
                it.ret(v("identity").call([int()]));
                it.var("identity", lambda(["x"], v("x")))
            }),
            "()",
        )
    }

    #[test]
    fn test_fix() {
        test(
            scope(|mut it| {
                it.var(
                    "fix",
                    lambda(
                        ["f"],
                        scope(|mut it| {
                            it.rec("x", v("f").call(["x"]));
                            it.ret(v("x"));
                        }),
                    ),
                );
                it.var("id", lambda(["x"], v("x")));
                it.ret(v("fix").call(["id"]));
            }),
            "∀a => a",
        )
    }

    #[test]
    fn test_fib() {
        test(
            scope(|mut it| {
                it.var(
                    "less",
                    foreign(
                        MonomorphicType::fun(
                            PrimitiveType::i8(),
                            [PrimitiveType::i8()],
                            PrimitiveType::bool().into(),
                        )
                        .generalize(&HashSet::new()),
                    ),
                );
                it.var(
                    "plus",
                    foreign(
                        MonomorphicType::fun(
                            PrimitiveType::i8(),
                            [PrimitiveType::i8()],
                            PrimitiveType::i8().into(),
                        )
                        .generalize(&HashSet::new()),
                    ),
                );
                it.rec(
                    "fib",
                    lambda(
                        ["n"],
                        scope(|mut it| {
                            it.ret(if_then_else(
                                v("less").call([v("n"), int()]),
                                int(),
                                v("plus").call([
                                    v("fib").call([v("plus").call([v("n"), int()])]),
                                    v("fib").call([v("plus").call([v("n"), int()])]),
                                ]),
                            ));
                        }),
                    ),
                );
                it.ret(v("fib"));
            }),
            "i8 -> i8",
        )
    }

    #[test]
    #[should_panic(expected = r#"Cannot typeck: [FailedConstraints(Cycle)]"#)]
    fn test_mutual_recursion() {
        // let rec is_odd = \x. not (is_even (pred x))
        // in
        //   let is_even = \y. not (is_odd (pred y))
        //   in
        //     is_odd

        test(
            scope(|mut it| {
                it.var("tuple2", lambda(["x", "y"], tuple([v("x"), v("y")])));
                it.var(
                    "not",
                    foreign(
                        MonomorphicType::fun1(PrimitiveType::bool(), PrimitiveType::bool())
                            .generalize(&HashSet::new()),
                    ),
                );
                it.var("one", lambda(["f", "x"], v("f").call(["x"])));
                it.var(
                    "plus",
                    lambda(
                        ["m", "n", "f", "x"],
                        v("m").call([v("f"), v("n").call(["f", "x"])]),
                    ),
                );
                it.var("succ", lambda(["x"], v("plus").call(["x", "one"])));
                it.rec(
                    "is_odd",
                    scope(|mut it| {
                        it.var(
                            "is_even",
                            lambda(
                                ["n"],
                                v("not").call([v("is_odd").call([v("succ").call(["n"])])]),
                            ),
                        );

                        it.ret(lambda(
                            ["n"],
                            v("not").call([v("is_even").call([v("succ").call(["n"])])]),
                        ));
                    }),
                );
                it.ret(v("is_odd"))
            }),
            "",
        )
    }

    #[test]
    #[should_panic(expected = r#"Cannot typeck: [UnknownName("id")]"#)]
    fn test_scoping() {
        test(
            scope(|mut it| {
                it.var("zero", lambda(["f", "x"], v("x")));
                it.add(scope(|mut it| {
                    it.var("id", lambda(["x"], v("x")));
                    it.ret(v("zero").call(["zero", "id"]));
                }));
                it.ret(v("zero").call(["zero", "id"]));
            }),
            "()",
        )
    }

    #[test]
    fn test_compose_fn() {
        test(
            scope(|mut it| {
                it.var(
                    "compose",
                    lambda(["f", "g"], lambda(["x"], v("f").call([v("g").call(["x"])]))),
                );
                it.ret(v("compose"))
            }),
            "∀a, b, c => (a -> b) -> (c -> a) -> c -> b",
        )
    }
}

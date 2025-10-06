// use crate::language::{app, lambda, r#let, var, Language};
// use crate::language::Literal;

// mod language;

// #[test]
// fn test_infer_language() {
//     // λz. let x = (z, z) in (λy. (y, y)) x
//     // ∀a, b, c => a -> ((a, a), (a, a))
//     let expr: Language = lambda(
//         "z",
//         r#let(
//             "x",
//             Literal::Tuple(vec![var("z").into(), var("z").into()]),
//             app(
//                 var("x"),
//                 lambda("y", Literal::Tuple(vec![var("y").into(), var("y").into()])),
//             ),
//         ),
//     )
//         .into();
//
//     let t = expr.infer(&Environment::empty()).unwrap();
//
//     println!("{}\n{}", expr, t);
//     assert_eq!(
//         t,
//         fun1(
//             t_var(0),
//             tuple([tuple([t_var(0), t_var(0)]), tuple([t_var(0), t_var(0)])])
//         )
//
//             .generalize(&HashSet::new())
//     );
// }
//
// #[test]
// fn test_church_encoding() {
//     //zero = \f. \x. x                   :: a -> b -> b
//     //one  = \f. \x. f x                 :: (a -> b) -> a -> b
//     //plus = \m. \n. \f. \x. m f (n f x) :: (a -> b -> c) -> (a -> d -> b) -> a -> d -> c
//
//     let zero: Language = lambda("f", lambda("x", var("x"))).into();
//     let one: Language = lambda("f", lambda("x", app(var("x"), var("f")))).into();
//     let plus: Language = lambda(
//         "m",
//         lambda(
//             "n",
//             lambda(
//                 "f",
//                 lambda(
//                     "x",
//                     app(
//                         app(var("x"), app(var("f"), var("n"))),
//                         app(var("f"), var("m")),
//                     ),
//                 ),
//             ),
//         ),
//     )
//         .into();
//
//     let zt = zero.infer(&Environment::empty()).unwrap();
//     let ot = one.infer(&Environment::empty()).unwrap();
//     let pt = plus.infer(&Environment::empty()).unwrap();
//
//     println!("{}\n{}\n\n{}\n{}\n\n{}\n{}", zero, zt, one, ot, plus, pt);
// }
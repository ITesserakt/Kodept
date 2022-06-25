package ru.tesserakt.kodept.traversal.inference.truely

sealed interface Expr<A> {
    data class App<A>(val a: Expr<A>, val b: Expr<A>) : Expr<A>
    data class Val<A>(val value: Value<A>) : Expr<A>
    data class Var<A>(val variable: A) : Expr<A>
    data class Let<A>(val a: Expr<A>, val b: Expr<A>, val c: Expr<A>) : Expr<A>
}

sealed interface Value<A> {
    data class Lambda<A>(val fn: (A) -> Expr<A>) : Value<A>
}

fun <A> Expr<Expr<A>>.substitute(): Expr<A> = when (this) {
    is Expr.App -> Expr.App(a.substitute(), b.substitute())
    is Expr.Let -> Expr.Let(a.substitute(), b.substitute(), c.substitute())
    is Expr.Val -> when (value) {
        is Value.Lambda -> Expr.Val(Value.Lambda { value.fn(Expr.Var(it)).substitute() })
    }
    is Expr.Var -> variable
}

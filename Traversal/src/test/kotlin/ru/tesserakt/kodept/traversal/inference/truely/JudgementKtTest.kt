package ru.tesserakt.kodept.traversal.inference.truely

import io.kotest.assertions.arrow.core.shouldBeLeft
import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.StringSpec
import using

class JudgementKtTest : StringSpec({
    "infer variable" {
        val typed = TExpr.TVar(0).toTypeLambda()

        using(Context(Type.Var(0)), TypeContext(0), Signature()) {
            typed infer Type.Binding(Type.Var(0))
        }.toEither().shouldBeRight()
    }

    "polymorphic const" {
        val type = TExpr.TVar(1).toLambda(Type.Var(1)).toLambda(Type.Var(0)).toTypeLambda().toTypeLambda()
        val expected = Type.Binding(Type.Binding(Type.Fn(Type.Var(0), Type.Fn(Type.Var(1), Type.Var(0)))))

        using(Context(), TypeContext(0), Signature()) {
            type infer expected
        }.toEither().shouldBeRight()
    }

    "wrong type" {
        val type = TExpr.TVar(0).toLambda(Type.Var(0)).toTypeLambda()
        val expected = Type.Binding(Type.Var(0))

        using(Context(), TypeContext(0), Signature()) {
            type infer expected
        }.toEither().shouldBeLeft()
    }
})

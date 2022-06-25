package ru.tesserakt.kodept.traversal.inference.truely

import ru.tesserakt.kodept.traversal.inference.truely.TExpr.*
import ru.tesserakt.kodept.traversal.inference.truely.TValue.*
import ru.tesserakt.kodept.traversal.inference.truely.Type.Binding
import ru.tesserakt.kodept.traversal.inference.truely.Type.Fn

class Judgements {
    private val assumes: MutableMap<Assume, Type> = mutableMapOf()

    infix fun TypeContextStep.withGiven(sigma: Signature) {
        assumes[Assume(this.context.type.assume.expr, this.context.context, this.type, sigma)] = this.context.type.type
    }

    infix fun EnsureTypeContextStep.withGiven(sigma: Signature) {
        require(
            assumes[Assume(
                this.context.typeStep.ensure.expr,
                this.context.context,
                this.type,
                sigma
            )] == this.context.typeStep.type
        )
    }

    fun assumeThat(expr: TExpr) = AssumeStep(expr)
    fun ensureThat(expr: TExpr) = EnsureStep(expr)
    fun ensureThat(condition: Boolean) = require(condition)

    companion object {
        fun typecheck(e: TExpr, t: Type, sigma: Signature) = with(Judgements()) {
            assumeThat(e) hasType t inContext Context() and TypeContext(0) withGiven sigma
        }

        fun Judgements.const(t: Type, c: Int, gamma: Context, delta: TypeContext, sigma: Signature) {
            ensureThat(sigma contains t at c)
            ensureThat(t isValidIn delta)
            assumeThat(TVal(TConst(c))) hasType t inContext gamma and delta withGiven sigma
        }

        fun Judgements.variable(t: Type, x: Int, gamma: Context, delta: TypeContext, sigma: Signature) {
            ensureThat(gamma contains t at x)
            ensureThat(t isValidIn delta)
            assumeThat(TVar(x)) hasType t inContext gamma and delta withGiven sigma
        }

        fun Judgements.application(
            t1: Type, t2: Type, e1: TExpr, e2: TExpr,
            gamma: Context, delta: TypeContext, sigma: Signature,
        ) {
            ensureThat(e1) hasType Fn(t1, t2) inContext gamma and delta withGiven sigma
            ensureThat(e2) hasType t1 inContext gamma and delta withGiven sigma
            assumeThat(TApp(e1, e2)) hasType t2 inContext gamma and delta withGiven sigma
        }

        fun Judgements.typeApplication(
            t1: Type,
            t2: Type,
            e: TExpr,
            gamma: Context,
            delta: TypeContext,
            sigma: Signature,
        ) {
            ensureThat(e) hasType Binding(t1) inContext gamma and delta withGiven sigma
            ensureThat(t2 isValidIn delta)
            assumeThat(TTypeApp(e, t2)) hasType t1.substitute(t2) inContext gamma and delta withGiven sigma
        }

        fun Judgements.abstraction(t1: Type, t2: Type, e: TExpr, gamma: Context, delta: TypeContext, sigma: Signature) {
            ensureThat(e) hasType t2 inContext (gamma + t1) and delta withGiven sigma
            assumeThat(TVal(TLambda(t1, e))) hasType Fn(t1, t2) inContext gamma and delta withGiven sigma
        }

        fun Judgements.typeAbstraction(t: Type, e: TExpr, gamma: Context, delta: TypeContext, sigma: Signature) {
            ensureThat(e) hasType t inContext gamma and delta.next() withGiven sigma
            assumeThat(TVal(TTypeLambda(e))) hasType Binding(t) inContext gamma and delta withGiven sigma
        }
    }
}
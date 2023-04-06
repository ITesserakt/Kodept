package ru.tesserakt.kodept.traversal.inference

import arrow.core.raise.EagerEffect
import arrow.core.raise.eagerEffect

sealed interface Errors {
    data class UnknownVariable(val variable: Language.Var) : Errors
    data class InfiniteType(val type: MonomorphicType.Var, val with: MonomorphicType) : Errors
    data class CannotUnify(val type: MonomorphicType, val with: MonomorphicType) : Errors
}

typealias AlgorithmWPureResult = Pair<Substitutions, MonomorphicType>
typealias AlgorithmWResult = EagerEffect<Errors, AlgorithmWPureResult>

class AlgorithmW(private val context: Assumptions, private val expression: Language) {
    fun Language.Var.run(): AlgorithmWResult = eagerEffect {
        val assumption = context[this@run]
        if (assumption != null) Substitution.empty() to assumption.instantiate()
        else raise(Errors.UnknownVariable(this@run))
    }

    fun Language.App.run(): AlgorithmWResult = eagerEffect {
        val beta = MonomorphicType.Var()
        val (s1, t1) = AlgorithmW(context, func).run().bind()
        val (s2, t2) = AlgorithmW(context.substitute(s1), arg).run().bind()
        val s3 = AlgorithmU(t1.substitute(s1), MonomorphicType.Fn(t2, beta)).run().bind()
        s3 compose s2 compose s1 to beta.substitute(s3)
    }

    fun Language.Lambda.run(): AlgorithmWResult = eagerEffect {
        val beta = MonomorphicType.Var()
        val (s1, t1) = AlgorithmW(context + (bind to beta), expr).run().bind()
        s1 to MonomorphicType.Fn(beta.substitute(s1), t1)
    }

    fun Language.Let.run(): AlgorithmWResult = eagerEffect {
        val (s1, t1) = AlgorithmW(context, binder).run().bind()
        val polyType = context.substitute(s1).generalize(t1)
        val newContext = Assumptions(context.filterKeys { it != bind }).substitute(s1)

        val (s2, t2) = AlgorithmW(newContext + (bind to polyType), usage).run().bind()
        s2 compose s1 to t2
    }

    fun Language.Literal.Tuple.run(): AlgorithmWResult = eagerEffect {
        items.fold(emptySet<Substitution>() to MonomorphicType.Tuple.unit) { acc, next ->
            val (s1, t1) = AlgorithmW(context.substitute(acc.first), next).run().bind()
            acc.first.compose(s1) to MonomorphicType.Tuple(acc.second.items + t1)
        }
    }

    fun run() = eagerEffect {
        when (expression) {
            is Language.App -> expression.run().bind()
            is Language.Lambda -> expression.run().bind()
            is Language.Let -> expression.run().bind()
            is Language.Var -> expression.run().bind()
            is Language.Literal.Tuple -> expression.run().bind()
            is Language.Literal.Floating -> Substitution.empty() to PrimitiveType.Floating
            is Language.Literal.Number -> Substitution.empty() to PrimitiveType.Number
        }
    }
}

infix fun Language.infer(context: Assumptions) = eagerEffect {
    val (s, t) = AlgorithmW(context, this@infer).run().bind()
    context.substitute(s).and(this@infer, context.generalize(t)) to t
}
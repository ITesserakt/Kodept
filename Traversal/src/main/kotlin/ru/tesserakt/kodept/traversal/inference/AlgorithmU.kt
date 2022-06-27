package ru.tesserakt.kodept.traversal.inference

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect

typealias AlgorithmUResult = EagerEffect<Errors, Substitutions>

class AlgorithmU(val a: MonomorphicType, val b: MonomorphicType) {
    private fun occursCheck(v: MonomorphicType.Var, type: MonomorphicType): Boolean = when {
        v == type -> true
        type is MonomorphicType.Fn -> occursCheck(v, type.input) || occursCheck(v, type.output)
        else -> false
    }

    fun run(): AlgorithmUResult = eagerEffect {
        when {
            a is MonomorphicType.Var && occursCheck(a, b) -> shift(Errors.InfiniteType(a, b))
            b is MonomorphicType.Var && occursCheck(b, a) -> shift(Errors.InfiniteType(b, a))
            a is MonomorphicType.Var -> Substitution(a, b).single()
            b is MonomorphicType.Var -> Substitution(b, a).single()
            a is MonomorphicType.Fn && b is MonomorphicType.Fn -> {
                val s1 = (a.input unify b.input).bind()
                val s2 = (a.output.substitute(s1) unify b.output.substitute(s1)).bind()
                s1 + s2
            }
            a is MonomorphicType.Tuple && b is MonomorphicType.Tuple && a.items.size == b.items.size -> {
                a.items.zip(b.items).fold(emptySet()) { acc, (x, y) ->
                    acc + (x.substitute(acc) unify y.substitute(acc)).bind()
                }
            }
            a == b -> emptySet()
            else -> shift(Errors.CannotUnify(a, b))
        }
    }
}

infix fun MonomorphicType.unify(other: MonomorphicType) = AlgorithmU(this, other).run()
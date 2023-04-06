package ru.tesserakt.kodept.traversal.inference

import arrow.core.raise.EagerEffect
import arrow.core.raise.eagerEffect

typealias AlgorithmUResult = EagerEffect<Errors, Substitutions>

class AlgorithmU(val a: MonomorphicType, val b: MonomorphicType) {
    private fun occursCheck(v: MonomorphicType.Var, type: MonomorphicType): Boolean = when {
        v == type -> true
        type is MonomorphicType.Fn -> occursCheck(v, type.input) || occursCheck(v, type.output)
        else -> false
    }

    fun run(): AlgorithmUResult = eagerEffect {
        when {
            a == b -> emptySet()
            a is MonomorphicType.Var && occursCheck(a, b) -> raise(Errors.InfiniteType(a, b))
            b is MonomorphicType.Var && occursCheck(b, a) -> raise(Errors.InfiniteType(b, a))
            a is MonomorphicType.Var -> Substitution(a, b).single()
            b is MonomorphicType.Var -> Substitution(b, a).single()
            a is MonomorphicType.Fn && b is MonomorphicType.Fn -> {
                val s1 = (a.input unify b.input).bind()
                val s2 = (a.output.substitute(s1) unify b.output.substitute(s1)).bind()
                s1 compose s2
            }
            a is MonomorphicType.Tuple && b is MonomorphicType.Tuple && a.items.size == b.items.size -> {
                a.items.zip(b.items).fold(emptySet()) { acc, (x, y) ->
                    acc compose (x.substitute(acc) unify y.substitute(acc)).bind()
                }
            }
            else -> raise(Errors.CannotUnify(a, b))
        }
    }
}

infix fun MonomorphicType.unify(other: MonomorphicType) = AlgorithmU(this, other).run()
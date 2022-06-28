package ru.tesserakt.kodept.traversal.inference

import arrow.core.NonEmptyList
import arrow.core.nonEmptyListOf

private fun Int.expandToString(alphabet: List<Char> = ('a'..'z').toList()): String {
    if (this == 0) return alphabet[0].toString()
    var current = this
    val sb = StringBuilder()
    while (current > 0) {
        sb.append(alphabet[current % alphabet.size])
        current /= alphabet.size
    }
    return sb.reverse().toString()
}

sealed class PrimitiveType : MonomorphicType() {
    object Number : PrimitiveType() {
        override fun toString() = ":number:"
    }

    object Floating : PrimitiveType() {
        override fun toString() = ":floating:"
    }
}

sealed class MonomorphicType : PolymorphicType() {
    data class Var(val id: Int) : MonomorphicType() {
        companion object {
            private var unique = 0
                get() = field++
        }

        constructor() : this(unique)

        fun new() = Var(unique)
        override fun toString(): String = id.expandToString()
    }

    data class Fn(val input: MonomorphicType, val output: MonomorphicType) : MonomorphicType() {
        override fun toString(): String = when (input) {
            is Fn -> "($input) -> $output"
            else -> "$input -> $output"
        }

        companion object {
            fun uncurry(args: NonEmptyList<MonomorphicType>, out: MonomorphicType) =
                args.dropLast(1).foldRight(Fn(args.last(), out), ::Fn)

            fun uncurry(head: MonomorphicType, vararg rest: MonomorphicType, out: MonomorphicType) =
                uncurry(nonEmptyListOf(head, *rest), out)
        }
    }

    data class Tuple(val items: List<MonomorphicType>) : MonomorphicType() {
        override fun toString(): String = items.joinToString(prefix = "(", postfix = ")")

        companion object {
            val unit = Tuple(emptyList())
        }
    }

    data class Constant(val id: Int) : MonomorphicType() {
        override fun toString(): String = id.expandToString(('Z'.downTo('A')).toList())
    }

    fun substitute(subst: Set<Substitution>): MonomorphicType = when (this) {
        is PrimitiveType -> this
        is Var -> subst.find { it.substituted == this }?.replacement ?: this
        is Fn -> Fn(input.substitute(subst), output.substitute(subst))
        is Tuple -> Tuple(items.map { it.substitute(subst) })
        is Constant -> this
    }

    fun freeTypes(): Set<Var> = when (this) {
        is Fn -> input.freeTypes().union(output.freeTypes())
        is Var -> setOf(this)
        is PrimitiveType -> emptySet()
        is Tuple -> items.fold(emptySet()) { acc, next -> acc + next.freeTypes() }
        is Constant -> emptySet()
    }

    fun rename(old: Int, new: Int): MonomorphicType = when (this) {
        is PrimitiveType -> this
        is Fn -> Fn(input.rename(old, new), output.rename(old, new))
        is Var -> if (id == old) Var(new) else this
        is Tuple -> Tuple(items.map { it.rename(old, new) })
        is Constant -> this
    }
}

sealed class PolymorphicType {
    abstract override fun toString(): String

    data class Binding(val bind: MonomorphicType.Var, val type: PolymorphicType) : PolymorphicType() {
        override fun toString(): String {
            val (bindings, t) = collect()
            val prettyBindings = bindings.asReversed().map { it.id }.zip(0..bindings.size)
            val renamed = prettyBindings.fold(t) { acc, (old, new) ->
                acc.rename(old, new)
            }
            return "∀${prettyBindings.joinToString { it.second.expandToString() }} => $renamed"
        }

        tailrec fun collect(
            acc: List<MonomorphicType.Var> = emptyList(),
            current: PolymorphicType = this,
        ): Pair<List<MonomorphicType.Var>, MonomorphicType> = when (current) {
            is MonomorphicType -> acc to current
            is Binding -> collect(acc + current.bind, current.type)
        }
    }

    fun instantiate(): MonomorphicType = when (this) {
        is MonomorphicType -> this
        is Binding -> when (type) {
            is MonomorphicType -> type.substitute(Substitution(bind, bind.new()).single())
            else -> type.instantiate()
        }
    }
}

fun PolymorphicType.substitute(subst: Set<Substitution>): PolymorphicType = when (this) {
    is MonomorphicType -> substitute(subst)
    is PolymorphicType.Binding -> PolymorphicType.Binding(
        bind,
        type.substitute(subst.filter { it.replacement != bind }.toSet())
    )
}

fun PolymorphicType.freeTypes(): Set<MonomorphicType.Var> = when (this) {
    is MonomorphicType -> freeTypes()
    is PolymorphicType.Binding -> type.freeTypes() subtract setOf(bind)
}
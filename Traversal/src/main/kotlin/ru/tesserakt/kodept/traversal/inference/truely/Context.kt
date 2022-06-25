package ru.tesserakt.kodept.traversal.inference.truely

@JvmInline
value class Context(val value: List<Type>) {
    operator fun plus(other: Type) = Context(value + other)

    data class ContainsStep(val s: Context, val t: Type) {
        infix fun at(pos: Int) = s.value.indexOf(t) == pos
    }

    infix fun contains(other: Type) = ContainsStep(this, other)

    constructor(vararg types: Type) : this(types.toList())
}

@JvmInline
value class TypeContext(val value: Int) {
    fun next() = TypeContext(value + 1)
    operator fun contains(other: Int) = other < value
}

@JvmInline
value class Signature(val value: List<Type>) {
    operator fun plus(other: Type) = Signature(value + other)

    data class ContainsStep(val s: Signature, val t: Type) {
        infix fun at(pos: Int) = s.value.indexOf(t) == pos
    }

    infix fun contains(other: Type) = ContainsStep(this, other)

    constructor(vararg types: Type) : this(types.toList())
}
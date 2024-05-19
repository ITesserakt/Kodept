package ru.tesserakt.kodept.traversal.inference

import arrow.core.flip
import arrow.typeclasses.Monoid

@JvmInline
value class Assumptions(private val value: Map<Language, PolymorphicType>) : Map<Language, PolymorphicType> by value {
    fun substitute(subst: Set<Substitution>) = Assumptions(value.mapValues { it.value.substitute(subst) })
    fun combine(other: Assumptions) = Assumptions(value + other.value)
    fun and(bind: Language, type: PolymorphicType) = Assumptions(value + (bind to type))
    operator fun plus(elem: Pair<Language, PolymorphicType>) = and(elem.first, elem.second)
    constructor(vararg values: Pair<Language, PolymorphicType>): this(values.toMap())

    companion object : Monoid<Assumptions> {
        override fun Assumptions.combine(b: Assumptions): Assumptions = this@combine.combine(b)

        override fun empty(): Assumptions = Assumptions(emptyMap())
    }
}

fun Assumptions.generalize(type: MonomorphicType): PolymorphicType {
    val vars = type.freeTypes() subtract entries.flatMap { it.value.freeTypes() }.toSet()
    return vars.fold(type as PolymorphicType, PolymorphicType::Binding.flip())
}

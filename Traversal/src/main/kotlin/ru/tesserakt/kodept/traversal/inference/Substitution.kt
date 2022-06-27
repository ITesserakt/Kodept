package ru.tesserakt.kodept.traversal.inference

data class Substitution(val substituted: MonomorphicType, val replacement: MonomorphicType) {
    companion object {
        fun empty() = emptySet<Substitution>()
    }

    fun single() = setOf(this)
}

typealias Substitutions = Set<Substitution>

operator fun Substitutions.plus(other: Substitutions) =
    other.map { Substitution(it.substituted, it.replacement.substitute(this)) }.union(this)

package ru.tesserakt.kodept.traversal.inference

data class Substitution(val substituted: MonomorphicType, val replacement: MonomorphicType) {
    companion object {
        fun empty() = emptySet<Substitution>()
    }

    fun single() = setOf(this)

    override fun toString(): String = "$substituted <~ {$replacement}"
}

typealias Substitutions = Set<Substitution>

infix fun Substitutions.compose(other: Substitutions) =
    other.map { Substitution(it.substituted, it.replacement.substitute(this)) }
        .union(this.map { Substitution(it.substituted, it.replacement.substitute(other)) })
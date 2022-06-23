package ru.tesserakt.kodept.traversal.inference

data class TypeScheme(val bounds: Set<Type.T>, val type: Type) {
    fun instantiate() = type.applySubstitutions(bounds.map { Substitution(it, Type.T()) }.toSet())

    fun applySubstitutions(substitutions: Substitutions) =
        TypeScheme(bounds, type.applySubstitutions(substitutions.filter { it.type in bounds }.toSet()))
}
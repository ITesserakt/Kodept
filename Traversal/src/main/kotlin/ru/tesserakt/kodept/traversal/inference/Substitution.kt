package ru.tesserakt.kodept.traversal.inference

data class Substitution(val type: Type, val replaceWith: Type)

infix fun Set<Substitution>.compose(other: Set<Substitution>) =
    other.map { Substitution(it.type, it.replaceWith.applySubstitutions(this)) }.union(this)
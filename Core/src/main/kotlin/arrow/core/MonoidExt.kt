package arrow.core

import arrow.typeclasses.Monoid

private object SetMonoid : Monoid<Set<Any?>> {
    override fun append(a: Set<Any?>, b: Set<Any?>): Set<Any?> = a + b

    override fun empty(): Set<Any?> = emptySet()
    override fun Set<Any?>.combine(b: Set<Any?>): Set<Any?> = this + b
}

@Suppress("UNCHECKED_CAST")
fun <T> Monoid.Companion.set() = SetMonoid as Monoid<Set<T>>
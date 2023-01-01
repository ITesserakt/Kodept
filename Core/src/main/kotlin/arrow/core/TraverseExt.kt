package arrow.core

import arrow.core.continuations.*

inline fun <R, T, V> Sequence<T>.traverse(crossinline f: context(Raise<R>) (T) -> V): EagerEffect<R, List<V>> = eagerEffect {
    buildList {
        this@traverse.forEach {
            add(f(this@eagerEffect, it))
        }
    }
}

fun <R, T> Sequence<EagerEffect<R, T>>.sequence(): EagerEffect<R, List<T>> = traverse { it() }

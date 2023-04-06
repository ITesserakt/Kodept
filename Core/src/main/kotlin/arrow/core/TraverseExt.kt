package arrow.core

import arrow.core.raise.EagerEffect
import arrow.core.raise.Raise
import arrow.core.raise.eagerEffect

inline fun <R, T, V> Sequence<T>.traverse(crossinline f: context(Raise<R>) (T) -> V): EagerEffect<R, List<V>> =
    eagerEffect {
        val raise = this
        buildList {
            this@traverse.forEach {
                add(f(raise, it))
            }
        }
    }

fun <R, T> Sequence<EagerEffect<R, T>>.sequence(): EagerEffect<R, List<T>> = traverse<R, _, _> { it() }

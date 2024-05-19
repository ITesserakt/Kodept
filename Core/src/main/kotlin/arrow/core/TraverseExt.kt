package arrow.core

import arrow.core.continuations.*

fun <R, T, V> Sequence<T>.traverse(f: (T) -> Effect<R, V>) = effect<R, List<V>> {
    buildList {
        this@traverse.forEach {
            add(f(it).bind())
        }
    }
}

fun <R, T, V> Sequence<T>.traverse(f: (T) -> EagerEffect<R, V>) = eagerEffect<R, List<V>> {
    buildList {
        this@traverse.forEach {
            add(f(it).bind())
        }
    }
}

fun <R, T, V> Sequence<T>.traverseEffect(f: suspend EagerEffectScope<R>.(T) -> V) = traverse { eagerEffect { f(it) } }

fun <R, T> Sequence<Effect<R, T>>.sequence() = traverse(::identity)

fun <R, T> Sequence<EagerEffect<R, T>>.sequence() = traverse(::identity)

fun <T, V> Sequence<T>.traverse(f: (T) -> Eval<V>) =
    buildList {
        this@traverse.forEach {
            add(f(it))
        }
    }.fold(Eval.now(emptyList<V>())) { accE, nextE ->
        nextE.flatMap { next -> accE.map { acc -> acc + next } }
    }

fun <T> Sequence<Eval<T>>.sequence() = traverse(::identity)

fun <T, V> Sequence<T>.lazyMap(f: (T) -> V) = traverse { Eval.later { f(it) } }

fun <T> Sequence<T>.lazyList() = lazyMap(::identity)
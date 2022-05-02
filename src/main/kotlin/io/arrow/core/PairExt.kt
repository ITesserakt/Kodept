@file:Suppress("NOTHING_TO_INLINE")

package io.arrow.core

inline fun <A, B, C> Pair<A, B>.map(f: (B) -> C) = first to f(second)

inline fun <A, B, C> Pair<A, B>.mapLeft(f: (A) -> C) = f(first) to second

inline fun <A, B, C, D> Pair<A, B>.bimap(f: (A) -> C, g: (B) -> D) = f(first) to g(second)

inline fun <A, B> fst(a: A, b: B) = a
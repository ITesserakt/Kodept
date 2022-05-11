@file:Suppress("NOTHING_TO_INLINE", "unused")

package io.arrow.core

import com.github.h0tk3y.betterParse.utils.Tuple1
import com.github.h0tk3y.betterParse.utils.Tuple2
import com.github.h0tk3y.betterParse.utils.Tuple3
import com.github.h0tk3y.betterParse.utils.Tuple4

inline fun <A, B, C> Pair<A, B>.map(f: (B) -> C) = first to f(second)

inline fun <A, B, C> Pair<A, B>.mapLeft(f: (A) -> C) = f(first) to second

inline fun <A, B, C, D> Pair<A, B>.bimap(f: (A) -> C, g: (B) -> D) = f(first) to g(second)

inline fun <A, B> fst(a: A, b: B) = a

inline fun <A, B> ((A) -> B).curry(): (Tuple1<A>) -> B = { this(it.t1) }
inline fun <A, B, C> ((A, B) -> C).curry(): (Tuple2<A, B>) -> C = { this(it.t1, it.t2) }
inline fun <A, B, C, D> ((A, B, C) -> D).curry(): (Tuple3<A, B, C>) -> D = { this(it.t1, it.t2, it.t3) }
inline fun <A, B, C, D, E> ((A, B, C, D) -> E).curry(): (Tuple4<A, B, C, D>) -> E = { this(it.t1, it.t2, it.t3, it.t4) }
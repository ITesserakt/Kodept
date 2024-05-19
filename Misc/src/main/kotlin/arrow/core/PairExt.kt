@file:Suppress("NOTHING_TO_INLINE", "unused")

package arrow.core

import com.github.h0tk3y.betterParse.utils.Tuple1
import com.github.h0tk3y.betterParse.utils.Tuple2
import com.github.h0tk3y.betterParse.utils.Tuple3
import com.github.h0tk3y.betterParse.utils.Tuple4

inline fun <A, B, C> Pair<A, B>.map(f: (B) -> C) = first to f(second)

inline fun <A, B, C> Pair<A, B>.mapLeft(f: (A) -> C) = f(first) to second

inline fun <A, B, C, D> Pair<A, B>.bimap(f: (A) -> C, g: (B) -> D) = f(first) to g(second)

inline fun <A, B> fst(a: A, @Suppress("UNUSED_PARAMETER") b: B) = a

inline fun <A, B> ((A) -> B).curry(): (Tuple1<A>) -> B = { this(it.t1) }
inline fun <A, B, C> ((A, B) -> C).curry(): (Tuple2<A, B>) -> C = { this(it.t1, it.t2) }
inline fun <A, B, C, D> ((A, B, C) -> D).curry(): (Tuple3<A, B, C>) -> D = { this(it.t1, it.t2, it.t3) }
inline fun <A, B, C, D, E> ((A, B, C, D) -> E).curry(): (Tuple4<A, B, C, D>) -> E = { this(it.t1, it.t2, it.t3, it.t4) }

inline fun <A, B, C> ((A, B) -> C).curryPair(): (Pair<A, B>) -> C = { this(it.first, it.second) }
inline fun <A, B, C, D> ((A, B, C) -> D).curryPair(): (Triple<A, B, C>) -> D = { this(it.first, it.second, it.third) }

inline fun <A, B> Tuple2<A, B>?.orNull() = this ?: Tuple2(null, null)
inline fun <A, B, C> Tuple2<A, Tuple2<B, C>>.flatten() = Tuple3(t1, t2.t1, t2.t2)
inline fun <A, B, C> Tuple3<A, B, C>?.orNull() = this ?: Tuple3(null, null, null)

@JvmName("orNullA?BC?")
inline fun <A, B, C> Tuple3<A, List<B>, C>?.orNull() = this ?: Tuple3(null, emptyList(), null)
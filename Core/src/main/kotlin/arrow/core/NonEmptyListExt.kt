@file:Suppress("unused")

package arrow.core

operator fun <T> NonEmptyList<T>.component1() = head
operator fun <T> NonEmptyList<T>.component2() = tail

fun <T> NonEmptyList<T>.pair() = head to tail
inline fun <T, U> NonEmptyList<T>.pairMap(f: (T) -> U) = f(head) to tail.map(f)
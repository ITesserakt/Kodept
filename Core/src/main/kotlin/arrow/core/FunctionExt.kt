package arrow.core

fun <T1, T2, T3> ((T1, T2) -> T3).flip() = { a: T2, b: T1 -> this(b, a) }
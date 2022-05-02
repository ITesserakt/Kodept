package io.arrow.core

fun <A, B, C> Pair<A, B>.map(f: (B) -> C) = first to f(second)

fun <A, B, C> Pair<A, B>.mapLeft(f: (A) -> C) = f(first) to second

fun <A, B, C, D> Pair<A, B>.bimap(f: (A) -> C, g: (B) -> D) = f(first) to g(second)
package arrow.core

import arrow.core.continuations.EagerEffectScope
import arrow.core.continuations.EffectScope
import kotlin.contracts.ExperimentalContracts
import kotlin.contracts.InvocationKind
import kotlin.contracts.contract

@OptIn(ExperimentalContracts::class)
suspend inline fun <A> EagerEffectScope<A>.ensureThat(condition: Boolean, f: () -> A) {
    contract {
        returns() implies condition
        callsInPlace(f, InvocationKind.EXACTLY_ONCE)
    }
    if (!condition) shift<Nothing>(f())
}

@OptIn(ExperimentalContracts::class)
suspend inline fun <A> EffectScope<A>.ensureThat(condition: Boolean, f: () -> A) {
    contract {
        returns() implies condition
        callsInPlace(f, InvocationKind.EXACTLY_ONCE)
    }
    if (!condition) shift<Nothing>(f())
}
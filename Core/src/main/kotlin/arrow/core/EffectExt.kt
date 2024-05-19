package arrow.core

import arrow.core.raise.Raise
import kotlin.contracts.ExperimentalContracts
import kotlin.contracts.InvocationKind
import kotlin.contracts.contract

@OptIn(ExperimentalContracts::class)
inline fun <E> Raise<E>.ensureThat(condition: Boolean, f: () -> E) {
    contract {
        returns() implies condition
        callsInPlace(f, InvocationKind.EXACTLY_ONCE)
    }
    if (!condition) raise(f())
}

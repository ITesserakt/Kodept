package ru.tesserakt.kodept.core

abstract class Contract<in T, C> {
    protected abstract fun T.implies(context: @UnsafeVariance C): String

    context (C, T)
            operator fun invoke(): Nothing = throw IllegalStateException("Contract violation: ${implies(this@C)}")
}

inline fun <T, C> Contract(crossinline block: T.(C) -> String) = object : Contract<T, C>() {
    override fun T.implies(context: C): String = block(context)
}
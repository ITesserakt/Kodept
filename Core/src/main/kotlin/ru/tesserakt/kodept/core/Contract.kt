package ru.tesserakt.kodept.core

abstract class Contract<in T, C> {
    protected abstract fun T.implies(context: C): String

    context (C, T)
            operator fun invoke(): Nothing = throw IllegalStateException("Contract violation: ${implies(this@C)}")
}

inline fun <T> Contract(crossinline block: T.() -> String) = object : Contract<T, Any?>() {
    override fun T.implies(context: Any?): String = block()
}
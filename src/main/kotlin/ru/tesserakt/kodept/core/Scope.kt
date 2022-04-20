package ru.tesserakt.kodept.core

sealed class Scope {
    abstract val module: String
    abstract infix fun isSuperScopeOf(other: Scope): Boolean
    abstract infix fun isSubScopeOf(other: Scope): Boolean

    data class Global(override val module: String) : Scope() {
        override fun isSubScopeOf(other: Scope): Boolean = this == other
        override fun isSuperScopeOf(other: Scope): Boolean = other.module == module
    }

    sealed class Inner<S : Scope> : Scope() {
        abstract val parent: S

        override fun isSubScopeOf(other: Scope): Boolean = this == other || parent.isSubScopeOf(other)
        override fun isSuperScopeOf(other: Scope): Boolean = when (other) {
            is Global -> false
            is Inner<*> -> other == this || isSuperScopeOf(other.parent)
        }
    }

    data class Object(override val parent: Global) : Inner<Global>() {
        override val module: String = parent.module
    }

    data class Local(override val parent: Inner<*>) : Inner<Inner<*>>() {
        override val module: String = parent.module
    }
}
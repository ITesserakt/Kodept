package ru.tesserakt.kodept.core

import java.util.*

sealed class Scope {
    abstract val module: String
    abstract infix fun isSuperScopeOf(other: Scope): Boolean
    abstract infix fun isSubScopeOf(other: Scope): Boolean
    abstract infix fun commonAncestor(other: Scope): Scope?

    data class Global(override val module: String) : Scope() {
        override fun isSubScopeOf(other: Scope): Boolean = this == other
        override fun isSuperScopeOf(other: Scope): Boolean = this.module == other.module
        override fun commonAncestor(other: Scope): Scope? = this.takeIf { this == other || this isSuperScopeOf other }
    }

    sealed class Inner<S : Scope> : Scope() {
        abstract val parent: S

        override fun isSubScopeOf(other: Scope): Boolean = this == other || parent.isSubScopeOf(other)
        override fun isSuperScopeOf(other: Scope): Boolean = when (other) {
            is Global -> false
            is Inner<*> -> other == this || this isSuperScopeOf other.parent
        }

        override fun commonAncestor(other: Scope): Scope? = when (other) {
            is Global -> other commonAncestor this
            is Inner<*> -> this.takeIf { this isSuperScopeOf other } ?: (parent commonAncestor other)
        }
    }

    data class Object(override val parent: Global, val id: UUID) : Inner<Global>() {
        override val module: String = parent.module
    }

    data class Local(override val parent: Inner<*>, val id: UUID) : Inner<Inner<*>>() {
        override val module: String = parent.module
    }
}
package ru.tesserakt.kodept.traversal.inference

import arrow.core.NonEmptyList

@Suppress("unused")
sealed interface KnownType {
    object BottomType : KnownType {
        override fun toString() = "ꓕ"
    }

    object Number : KnownType {
        override fun toString() = ":number:"
    }

    object Char : KnownType {
        override fun toString() = ":char:"
    }

    object String : KnownType {
        override fun toString() = ":string:"
    }

    object Floating : KnownType {
        override fun toString() = ":floating:"
    }

    /**
     * Will be replaced after unifying, represents pure type without any constraints
     */
    data class T(val name: kotlin.String) : KnownType {
        override fun toString() = name
    }

    /**
     * Represents an unsorted list of types
     */
    data class Tuple(val content: List<KnownType>) : KnownType {
        override fun toString() = content.joinToString(", ", "(", ")")
    }

    data class Union(val items: NonEmptyList<KnownType>) : KnownType {
        override fun toString() = items.joinToString(" | ", "(", ")")
    }

    data class Fn(val input: List<KnownType>, val output: KnownType) : KnownType {
        override fun toString() = "(${input.joinToString(", ")}) -> $output"
    }

    data class Struct(val name: kotlin.String, val items: List<KnownType>) : KnownType {
        override fun toString() = "$name${items.joinToString(", ", " {", "}")}"
    }

    data class Interface(val name: kotlin.String) : KnownType {
        override fun toString() = "Trait $name"
    }

    data class Enum(val name: kotlin.String, val entries: List<KnownType>) : KnownType {
        override fun toString(): kotlin.String = "$name${entries.joinToString(" | ", "{", "}")}"
    }

    companion object {
        fun Tag(name: kotlin.String) = Struct(name, emptyList())
        val unit = Tuple(emptyList())
    }
}
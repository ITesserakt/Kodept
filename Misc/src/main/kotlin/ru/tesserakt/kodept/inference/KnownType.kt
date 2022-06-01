package ru.tesserakt.kodept.inference

import arrow.core.NonEmptyList
import ru.tesserakt.kodept.core.RLT

@Suppress("unused")
sealed interface KnownType {
    object BottomType : KnownType
    object Number : KnownType
    object Bool : KnownType
    object Char : KnownType
    object String : KnownType
    object Floating : KnownType

    /**
     * Will be replaced after unifying, represents pure type without any constraints
     */
    data class T(val name: kotlin.String) : KnownType

    /**
     * Represents an unsorted list of types
     */
    data class Tuple(val content: List<KnownType>) : KnownType {
        fun flatten(): Tuple = content.fold(emptyList<KnownType>()) { acc, next ->
            when (next) {
                is Tuple -> acc + next.flatten().content
                else -> acc + next
            }
        }.let(::Tuple)
    }

    data class Union(val items: NonEmptyList<KnownType>) : KnownType
    data class Fn(val input: List<KnownType>, val output: KnownType) : KnownType
    data class Struct(val name: kotlin.String, val items: List<KnownType>) : KnownType
    data class Interface(val name: kotlin.String) : KnownType

    companion object {
        fun Tag(name: kotlin.String) = Struct(name, emptyList())
        val unit = Tuple(emptyList())
    }
}

fun RLT.TypeNode.known(): KnownType = when (this) {
    is RLT.TupleType -> KnownType.Tuple(types.map { it.known() })
    is RLT.UserSymbol.Type -> KnownType.T(text.value())
    is RLT.UnionType -> KnownType.Union(types.map { it.known() })
}
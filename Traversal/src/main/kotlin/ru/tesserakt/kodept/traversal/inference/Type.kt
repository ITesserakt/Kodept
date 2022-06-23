package ru.tesserakt.kodept.traversal.inference

private fun Int.expandToString(alphabet: List<Char> = ('a'..'z').toList()): String {
    if (this == 0) return alphabet[0].toString()
    var current = this
    val sb = StringBuilder()
    while (current > 0) {
        sb.append(alphabet[current % alphabet.size])
        current /= alphabet.size
    }
    return sb.reverse().toString()
}

sealed interface Type {
    data class T(val id: Id) : Type {
        @JvmInline
        value class Id(private val id: Int) {
            val name get() = "`${id.expandToString()}"
        }

        override fun toString() = id.name

        companion object {
            private var uniqueId = 0
                get() = field++

            operator fun invoke() = T(Id(uniqueId))
        }
    }

    data class Tuple(val content: List<Type>) : Type {
        constructor(vararg items: Type) : this(items.toList())

        override fun toString() = content.joinToString(", ", "(", ")")
    }

    data class Union(val items: List<Type>) : Type {
        override fun toString() = items.joinToString(" | ", "(", ")")
    }

    data class Fn(val input: Type, val output: Type) : Type {
        override fun toString() = "$input -> ($output)"

        companion object {
            fun fromParams(params: List<Type>, ret: Type) =
                if (params.isNotEmpty())
                    params.foldRight(ret) { next, acc -> Fn(next, acc) }
                else Fn(unit, ret)
        }
    }

    data class Struct(val name: String, val items: Lazy<List<Type>>, val inheritFrom: Lazy<Type>? = null) : Type {
        override fun toString() =
            "$name${if (items.value.isNotEmpty()) items.value.joinToString(", ", " {", "}") else ""}"

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as Struct

            if (name != other.name) return false
            if (items != other.items) return false

            return true
        }

        override fun hashCode(): Int {
            var result = name.hashCode()
            result = 31 * result + items.hashCode()
            return result
        }
    }

    data class Interface(val name: String) : Type {
        override fun toString() = "Trait $name"
    }

    data class Enum(val name: String, val entries: List<Type>) : Type {
        override fun toString(): String = "$name${entries.joinToString(" | ", "{", "}")}"
    }

    object Bottom : Type {
        override fun toString(): String = "ꓕ"
    }

    companion object {
        fun tag(name: String, inheritFrom: Lazy<Type>? = null) = Struct(name, lazyOf(emptyList()), inheritFrom)
        val bool = Enum("Bool", listOf(tag("True"), tag("False")))
        val number = tag("Int")
        val floating = tag("Double")
        val string = tag("String")
        val char = tag("Char")
        val unit = Tuple(emptyList())
    }

    fun applySubstitutions(substitutions: Set<Substitution>): Type = when (this) {
        is T -> substitutions.find { it.type == this }?.replaceWith ?: this
        is Fn -> Fn(input.applySubstitutions(substitutions), output.applySubstitutions(substitutions))
        is Struct -> substitutions.find { it.type == this }?.replaceWith ?: this
        else -> this
    }
}


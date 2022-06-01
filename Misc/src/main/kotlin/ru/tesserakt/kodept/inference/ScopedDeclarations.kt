package ru.tesserakt.kodept.inference

import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.inference.ScopedDeclarations.Kind.*

data class ScopedDeclarations<K : ScopedDeclarations.Kind>(
    val root: RLT.Scoping,
    val declarations: List<RLT.Bind>,
    val parent: ScopedDeclarations<*>?,
    val kind: K,
) {
    var typesInUse = 0

    val nearestGlobalOrObject: ScopedDeclarations<*>
        get() = takeIf { it.kind is Kind.Global || it.kind is Kind.Object } ?: parent!!.nearestGlobalOrObject

    private fun findInRoot(ref: RLT.Reference) = (root as? RLT.Named)
        ?.takeIf { ref.ref is RLT.UserSymbol.Type }
        ?.takeIf { ref.ref == it.id }

    fun findByReference(ref: RLT.Reference): RLT.Named? = declarations.mapNotNull { node ->
        if (ref.ref !is RLT.UserSymbol.Identifier) return@mapNotNull null
        if (node is RLT.Named) node.takeIf { it.id == ref.ref }
        else null
    }.firstOrNull() ?: findInRoot(ref) ?: if (kind == Kind.Protected)
        parent?.nearestGlobalOrObject?.findByReference(ref)
    else parent?.findByReference(ref)

    /**
     * [Global] - contains structs, traits, functions, enums in one module
     *
     * [Object] - contains variables in structs and struct
     *
     * [Protected] - contains functions and their parameters
     *
     * [Local] - contains blocks or single expressions
     */
    sealed interface Kind {
        sealed interface NonProtected : Kind
        sealed interface InsideBlock : Kind

        object Global : NonProtected
        object Object : NonProtected
        object Protected : InsideBlock
        object Local : NonProtected, InsideBlock
    }
}

@Suppress("UNCHECKED_CAST")
val ScopedDeclarations<ScopedDeclarations.Kind.Local>.refined get() = declarations as List<RLT.BlockLevelNode>
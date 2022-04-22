package ru.tesserakt.kodept.core

import ru.tesserakt.kodept.core.Scope as RealScope

class MetadataStore(private val delegate: Set<Key> = emptySet()) : Set<MetadataStore.Key> by delegate {
    sealed interface Key {
        sealed interface Unique : Key {
            override val unique get() = true
        }

        sealed interface Required : Unique

        val unique: Boolean

        @JvmInline
        value class Scope(val value: RealScope) : Required {
            operator fun invoke() = value
        }

        @JvmInline
        value class TermDescriptor(val value: AST.Node) : Required {
            operator fun invoke() = value
        }
    }

    inline fun <reified K : Key.Required> retrieveRequired() = retrieve<K>()
        ?: throw IllegalStateException("Tried to get required data ${
            K::class.simpleName
        } from store, but corresponding processor was not fulfilled")

    inline fun <reified K : Key.Unique> retrieve() = retrieveMany<K>().firstOrNull()

    inline fun <reified K : Key> retrieveMany() = filterIsInstance<K>()

    operator fun plus(element: Key): MetadataStore = MetadataStore(delegate + element)

    override fun equals(other: Any?) = other is MetadataStore && delegate == other.delegate

    override fun hashCode() = delegate.hashCode()

    override fun toString() = delegate.toString()
}

fun emptyStore() = MetadataStore()

fun RealScope.toKey() = MetadataStore.Key.Scope(this)

fun <N : AST.Node> N.appendMetadata(item: MetadataStore.Key): MetadataStore =
    if (!item.unique || metadata.filterIsInstance<MetadataStore.Key.Scope>().isEmpty())
        metadata + item
    else
        throw IllegalArgumentException("Trying to add second instance $item of unique data ${item::class.simpleName}")

private fun AST.Node.retrieveScope() = metadata.retrieveRequired<MetadataStore.Key.Scope>().value

val AST.ModuleDecl.scope get() = retrieveScope() as RealScope.Global
val AST.FileDecl.scope get() = retrieveScope() as RealScope.Global
val AST.FunctionDecl.scope get() = retrieveScope() as RealScope.Inner<*>
val AST.TopLevelDecl.scope get() = retrieveScope() as RealScope.Object
val AST.Node.scope get() = retrieveScope()

val AST.Term.descriptor get() = metadata.retrieveRequired<MetadataStore.Key.TermDescriptor>()
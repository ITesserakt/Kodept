package ru.tesserakt.kodept.core

import ru.tesserakt.kodept.parser.RLT

class MetadataStore(private val delegate: MutableSet<Key> = mutableSetOf()) :
    MutableSet<MetadataStore.Key> by delegate {
    sealed interface Key {
        sealed interface Unique : Key {
            override val unique get() = true
        }

        sealed interface Required : Unique

        val unique: Boolean

        @JvmInline
        value class RLTReference(val value: RLT.Node) : Required {
            @Suppress("UNCHECKED_CAST")
            operator fun <T : RLT.Node> invoke() = value as T
        }

        @JvmInline
        value class Referral(val value: AST.Referable) : Unique {
            operator fun invoke() = value
        }
    }

    inline fun <reified K : Key.Required> retrieveRequired() = retrieve<K>()
        ?: throw IllegalStateException(
            "Tried to get required data ${
                K::class.simpleName
            } from store, but corresponding processor was not fulfilled"
        )

    inline fun <reified K : Key.Unique> retrieve() = retrieveMany<K>().firstOrNull()

    inline fun <reified K : Key> retrieveMany() = filterIsInstance<K>()

    inline operator fun <reified K : Key> plus(element: K): MetadataStore =
        if (element !is Key.Unique || filterIsInstance<K>().isEmpty())
            MetadataStore(((this as MutableSet<Key>) + element).toMutableSet())
        else
            throw IllegalArgumentException("Trying to add second instance $element of unique data ${element::class.simpleName}")

    inline operator fun <reified K : Key> plusAssign(element: K) {
        if (element !is Key.Unique || filterIsInstance<K>().isEmpty())
            (this as MutableSet<Key>).plusAssign(element)
        else
            throw IllegalArgumentException("Trying to add second instance $element of unique data ${element::class.simpleName}")
    }

    override fun equals(other: Any?) = other is MetadataStore && delegate == other.delegate

    override fun hashCode() = delegate.hashCode()

    override fun toString() = delegate.toString()
}

fun emptyStore() = MetadataStore()

fun RLT.Node.wrap() = MetadataStore.Key.RLTReference(this)
fun AST.Referable.wrap() = MetadataStore.Key.Referral(this)

private inline fun <reified R : RLT.Node> AST.Node.retrieveRLTNode() =
    metadata.retrieveRequired<MetadataStore.Key.RLTReference>().value as R

val AST.FileDecl.rlt get() = retrieveRLTNode<RLT.File>()
val AST.ModuleDecl.rlt get() = retrieveRLTNode<RLT.Module>()
val AST.StructDecl.rlt get() = retrieveRLTNode<RLT.Struct>()
val AST.TraitDecl.rlt get() = retrieveRLTNode<RLT.Trait>()
val AST.EnumDecl.rlt get() = retrieveRLTNode<RLT.Enum>()
val AST.EnumDecl.Entry.rlt get() = retrieveRLTNode<RLT.UserSymbol.Type>()
val AST.FunctionDecl.rlt get() = retrieveRLTNode<RLT.Function.Bodied>()
val AST.AbstractFunctionDecl.rlt get() = retrieveRLTNode<RLT.Function.Abstract>()
val AST.StringLiteral.rlt get() = retrieveRLTNode<RLT.Literal.Text>()
val AST.CharLiteral.rlt get() = retrieveRLTNode<RLT.Literal.Text>()
val AST.DecimalLiteral.rlt get() = retrieveRLTNode<RLT.Literal.Floating>()
val AST.FloatingLiteral.rlt get() = retrieveRLTNode<RLT.Literal.Floating>()
val AST.BinaryLiteral.rlt get() = retrieveRLTNode<RLT.Literal.Number>()
val AST.OctalLiteral.rlt get() = retrieveRLTNode<RLT.Literal.Number>()
val AST.HexLiteral.rlt get() = retrieveRLTNode<RLT.Literal.Number>()
val AST.VariableDecl.rlt get() = retrieveRLTNode<RLT.Variable>()
val AST.InitializedVar.rlt get() = retrieveRLTNode<RLT.Assignment>()
val AST.Assignment.rlt get() = retrieveRLTNode<RLT.Assignment>()
val AST.Reference.rlt get() = retrieveRLTNode<RLT.Reference>()
val AST.TypeReference.rlt get() = retrieveRLTNode<RLT.Reference>()
val AST.FunctionCall.rlt get() = retrieveRLTNode<RLT.Application>()
val AST.Dereference.rlt get() = retrieveRLTNode<RLT.BinaryOperation>()
val AST.ExpressionList.rlt get() = retrieveRLTNode<RLT.Body.Block>()
val AST.TypeExpression.rlt get() = retrieveRLTNode<RLT.TypeNode>()
val AST.Type.rlt get() = retrieveRLTNode<RLT.UserSymbol.Type>()
val AST.TupleType.rlt get() = retrieveRLTNode<RLT.TupleType>()
val AST.UnionType.rlt get() = retrieveRLTNode<RLT.UnionType>()
val AST.IfExpr.rlt get() = retrieveRLTNode<RLT.If>()
val AST.IfExpr.ElifExpr.rlt get() = retrieveRLTNode<RLT.If.Elif>()
val AST.IfExpr.ElseExpr.rlt get() = retrieveRLTNode<RLT.If.Else>()
val AST.WhileExpr.rlt get() = retrieveRLTNode<RLT.While>()
val AST.Node.rlt get() = retrieveRLTNode<RLT.Node>()

val AST.Reference.referral get() = metadata.retrieve<MetadataStore.Key.Referral>()
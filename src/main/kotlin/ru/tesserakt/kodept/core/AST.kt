package ru.tesserakt.kodept.core

import arrow.core.NonEmptyList
import arrow.core.identity
import arrow.core.prependTo
import java.math.BigDecimal
import java.math.BigInteger

data class AST(val root: Node, val filename: Filename) {
    init {
        walkThrough { node -> node.children().forEach { it.parent = node } }
    }

    inline fun <T> walkThrough(mode: Tree.SearchMode = Tree.SearchMode.LevelOrder, crossinline f: (Node) -> T) =
        root.walkTopDown(mode, f)

    fun flatten(mode: Tree.SearchMode = Tree.SearchMode.LevelOrder) = root.gatherChildren(mode)

    sealed class Node : Tree<Node>() {
        final override var parent: Node? = null
        val metadata: MetadataStore = emptyStore()

        fun gatherChildren(mode: SearchMode = SearchMode.LevelOrder) = walkTopDown(mode, ::identity)
    }

    sealed class Leaf : Node() {
        final override fun children() = emptyList<Node>()
    }

    data class Parameter(var name: String, var type: TypeExpression) : Node() {
        override fun children() = listOf(type)
    }

    data class InferredParameter(var name: String, var type: TypeExpression?) : Node() {
        override fun children() = listOfNotNull(type)
    }

    data class FileDecl(var modules: NonEmptyList<ModuleDecl>) : Node() {
        override fun children() = modules
    }

    data class ModuleDecl(var name: String, var global: Boolean, var rest: List<Node>) : Node() {
        override fun children() = rest
    }

    data class StructDecl(
        var name: String, var alloc: List<Parameter>, var rest: List<Node>,
    ) : Node() {
        override fun children() = alloc + rest
    }

    data class EnumDecl(
        var name: String, var stackBased: Boolean, var enumEntries: List<Entry>,
    ) : Node() {
        override fun children() = enumEntries

        data class Entry(var name: String) : Leaf()
    }

    data class TraitDecl(var name: String, var rest: List<Node>) : Node() {
        override fun children() = rest
    }

    data class AbstractFunctionDecl(
        var name: String, var params: List<Parameter>, var returns: TypeExpression?,
    ) : Node() {
        override fun children() = params + listOfNotNull(returns)
    }

    data class FunctionDecl(
        var name: String,
        var params: List<InferredParameter>,
        var returns: TypeExpression?,
        var rest: Node,
    ) : Node() {
        override fun children() = params + listOf(rest) + listOfNotNull(returns)
    }

    data class VariableDecl(
        var name: String, var mutable: Boolean, var type: TypeExpression?,
    ) : Node() {
        override fun children() = listOfNotNull(type)
    }

    data class InitializedVar(var decl: VariableDecl, var expr: Node) : Node() {
        override fun children() = listOf(decl, expr)
    }

    data class DecimalLiteral(var value: BigInteger) : Leaf()
    data class BinaryLiteral(var value: BigInteger) : Leaf()
    data class OctalLiteral(var value: BigInteger) : Leaf()
    data class HexLiteral(var value: BigInteger) : Leaf()
    data class CharLiteral(var value: Char) : Leaf()
    data class StringLiteral(var value: String) : Leaf()
    data class FloatingLiteral(var value: BigDecimal) : Leaf()

    data class TupleLiteral(var items: List<Node>) : Node() {
        override fun children() = items

        @Suppress("unused")
        val arity = items.size

        companion object {
            val unit = TupleLiteral(emptyList())
        }
    }

    data class Mathematical(var left: Node, var right: Node, var kind: Kind) : Node() {
        override fun children() = listOf(left, right)

        enum class Kind { Add, Sub, Mul, Div, Mod, Pow }
    }

    data class Logical(var left: Node, var right: Node, var kind: Kind) : Node() {
        override fun children() = listOf(left, right)

        enum class Kind { Conjunction, Disjunction }
    }

    data class Comparison(var left: Node, var right: Node, var kind: Kind) : Node() {
        override fun children() = listOf(left, right)

        enum class Kind { Less, LessEqual, Equal, NonEqual, GreaterEqual, Greater, Complex }
    }

    data class Binary(var left: Node, var right: Node, var kind: Kind) : Node() {
        override fun children() = listOf(left, right)

        enum class Kind { And, Or, Xor }
    }

    data class Negation(var expr: Node) : Node() {
        override fun children() = listOf(expr)
    }

    data class Inversion(var expr: Node) : Node() {
        override fun children() = listOf(expr)
    }

    data class BitInversion(var expr: Node) : Node() {
        override fun children() = listOf(expr)
    }

    data class Absolution(var expr: Node) : Node() {
        override fun children() = listOf(expr)
    }

    data class Elvis(var left: Node, var right: Node) : Node() {
        override fun children() = listOf(left, right)
    }

    data class Assignment(var left: Node, var right: Node) : Node() {
        override fun children() = listOf(left, right)
    }

    data class ResolutionContext(var fromRoot: Boolean, var chain: List<TypeReference>)

    data class Reference(
        var name: String,
        var resolutionContext: ResolutionContext? = null,
    ) : Leaf()

    data class TypeReference(
        var type: TypeExpression, var resolutionContext: ResolutionContext? = null,
    ) : Node() {
        override fun children() = listOf(type)
        var name: String = type.type
    }

    data class FunctionCall(
        var reference: Node,
        var params: List<Node>,
        var resolutionContext: ResolutionContext? = null,
    ) : Node() {
        override fun children() = reference.prependTo(params)
    }

    data class TermChain(var terms: NonEmptyList<Node>) : Node() {
        override fun children() = terms
    }

    data class ExpressionList(var expressions: List<Node>) : Node() {
        override fun children() = expressions
    }

    data class TypeExpression(var type: String) : Leaf()

    data class IfExpr(
        var condition: Node, var body: Node, var elifs: List<ElifExpr>, var el: ElseExpr?,
    ) : Node() {
        override fun children() = listOf(condition, body) + elifs + listOfNotNull(el)

        data class ElifExpr(var condition: Node, var body: Node) : Node() {
            override fun children() = listOf(condition, body)
        }

        data class ElseExpr(var body: Node) : Node() {
            override fun children() = listOf(body)
        }
    }

    data class WhileExpr(var condition: Node, var body: Node) : Node() {
        override fun children() = listOf(condition, body)
    }
}
package ru.tesserakt.kodept.core

import arrow.core.NonEmptyList
import arrow.core.prependTo
import java.math.BigDecimal
import java.math.BigInteger

data class AST(val root: Node, val fileName: String) {
    sealed interface Node {
        val children: List<Node>
        val metadata: MetadataStore
    }

    sealed interface Leaf : Node {
        override val children: List<Node> get() = emptyList()
    }

    sealed interface TopLevelDecl : Node

    sealed interface ObjectLevelDecl : Node

    sealed interface BlockLevelDecl : Node

    sealed interface NamedDecl : Node {
        val name: String
    }

    sealed interface TypedDecl : Node {
        val type: String
    }

    sealed class CallableDecl : Node

    sealed class ObjectDecl : Node

    sealed class Expression : Node, BlockLevelDecl

    sealed class Literal : Expression() {
        sealed class Number : Literal()
    }

    sealed class Operation : Expression() {
        sealed class Binary : Operation() {
            abstract val left: Expression
            abstract val right: Expression
            override val children: List<Node> get() = listOf(left, right)
        }

        sealed class Unary : Operation() {
            abstract val expr: Expression
            override val children: List<Node> get() = listOf(expr)
        }
    }

    sealed class Term : Expression() {
        sealed class Simple : Term()
    }

    sealed class CodeFlowExpr : Expression()

    data class Parameter(
        override val name: String,
        val type: TypeExpression,
        override val metadata: MetadataStore = emptyStore(),
    ) : Leaf, NamedDecl {
        override val children get() = listOf(type)
    }

    data class InferredParameter(
        override val name: String,
        val type: TypeExpression?,
        override val metadata: MetadataStore = emptyStore(),
    ) : Leaf, NamedDecl {
        override val children get() = listOfNotNull(type)
    }

    data class FileDecl(
        val modules: NonEmptyList<ModuleDecl>,
        override val metadata: MetadataStore = emptyStore(),
    ) : Node {
        override val children get() = modules
    }

    data class ModuleDecl(
        override val name: String,
        val global: Boolean,
        val rest: List<TopLevelDecl>,
        override val metadata: MetadataStore = emptyStore(),
    ) : NamedDecl {
        override val children get() = rest
    }

    data class StructDecl(
        override val name: String,
        val alloc: List<Parameter>,
        val rest: List<ObjectLevelDecl>,
        override val metadata: MetadataStore = emptyStore(),
    ) : ObjectDecl(), TopLevelDecl, NamedDecl {
        override val children get() = alloc + rest
    }

    data class EnumDecl(
        override val name: String,
        val stackBased: Boolean,
        val enumEntries: List<Entry>,
        override val metadata: MetadataStore = emptyStore(),
    ) : ObjectDecl(), TopLevelDecl, NamedDecl {
        override val children get() = enumEntries

        data class Entry(
            override val name: String,
            override val metadata: MetadataStore = emptyStore(),
        ) : ObjectDecl(), Leaf, NamedDecl
    }

    data class TraitDecl(
        override val name: String,
        val rest: List<ObjectLevelDecl>,
        override val metadata: MetadataStore = emptyStore(),
    ) : ObjectDecl(), TopLevelDecl, NamedDecl {
        override val children get() = rest
    }

    data class AbstractFunctionDecl(
        override val name: String,
        val params: List<Parameter>,
        val returns: TypeExpression?,
        override val metadata: MetadataStore = emptyStore(),
    ) : ObjectLevelDecl, NamedDecl {
        override val children get() = params + listOfNotNull(returns)
    }

    data class FunctionDecl(
        override val name: String,
        val params: List<InferredParameter>,
        val returns: TypeExpression?,
        val rest: Expression,
        override val metadata: MetadataStore = emptyStore(),
    ) : CallableDecl(), TopLevelDecl, NamedDecl, ObjectLevelDecl, BlockLevelDecl {
        override val children get() = params + listOf(rest) + listOfNotNull(returns)
    }

    data class VariableDecl(
        override val name: String,
        val mutable: Boolean,
        val type: TypeExpression?,
        override val metadata: MetadataStore = emptyStore(),
    ) : CallableDecl(), NamedDecl, BlockLevelDecl {
        override val children get() = listOfNotNull(type)
    }

    data class InitializedVar(
        val decl: VariableDecl,
        val expr: Expression,
        override val metadata: MetadataStore = emptyStore(),
    ) : CallableDecl(), NamedDecl by decl,
        BlockLevelDecl by decl {
        override val children get() = listOf(decl, expr)
    }

    data class DecimalLiteral(
        val value: BigInteger,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal.Number(), Leaf

    data class BinaryLiteral(
        val value: BigInteger,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal.Number(), Leaf

    data class OctalLiteral(
        val value: BigInteger,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal.Number(), Leaf

    data class HexLiteral(
        val value: BigInteger,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal.Number(), Leaf

    data class CharLiteral(
        val value: Char,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal(), Leaf

    data class StringLiteral(
        val value: String,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal(), Leaf

    data class FloatingLiteral(
        val value: BigDecimal,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal.Number(), Leaf

    data class TupleLiteral(
        val items: List<Expression>,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal() {
        override val children = items

        val arity = items.size

        companion object {
            val unit = TupleLiteral(emptyList())
        }
    }

    data class Mathematical(
        override val left: Expression,
        override val right: Expression,
        val kind: Kind,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary() {
        enum class Kind { Add, Sub, Mul, Div, Mod, Pow }
    }

    data class Logical(
        override val left: Expression,
        override val right: Expression,
        val kind: Kind,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary() {
        enum class Kind { Conjunction, Disjunction }
    }

    data class Comparison(
        override val left: Expression,
        override val right: Expression,
        val kind: Kind,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary() {
        enum class Kind { Less, LessEqual, Equal, NonEqual, GreaterEqual, Greater, Complex }
    }

    data class Binary(
        override val left: Expression,
        override val right: Expression,
        val kind: Kind,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary() {
        enum class Kind { And, Or, Xor }
    }

    data class Negation(
        override val expr: Expression,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Unary()

    data class Inversion(
        override val expr: Expression,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Unary()

    data class BitInversion(
        override val expr: Expression,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Unary()

    data class Absolution(
        override val expr: Expression,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Unary()

    data class Elvis(
        override val left: Expression,
        override val right: Expression,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary()

    data class Assignment(
        override val left: Term,
        override val right: Expression,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary()

    data class ResolutionContext(val fromRoot: Boolean, val chain: List<TypeReference>)

    data class Reference(
        override val name: String,
        val resolutionContext: ResolutionContext? = null,
        override val metadata: MetadataStore = emptyStore(),
    ) : Term.Simple(), NamedDecl, Leaf

    data class TypeReference(
        val type: TypeExpression,
        val resolutionContext: ResolutionContext? = null,
    ) : Term.Simple(), NamedDecl, Leaf {
        override val name: String = type.type
        override val metadata: MetadataStore = type.metadata
    }

    data class FunctionCall(
        val reference: Expression,
        val params: List<Expression>,
        val resolutionContext: ResolutionContext? = null,
        override val metadata: MetadataStore = emptyStore(),
    ) : Term.Simple() {
        override val children get() = reference.prependTo(params)
    }

    data class TermChain(
        val terms: NonEmptyList<Expression>,
        override val metadata: MetadataStore = emptyStore(),
    ) : Term() {
        override val children get() = terms
    }

    data class ExpressionList(
        val expressions: List<BlockLevelDecl>,
        override val metadata: MetadataStore = emptyStore(),
    ) : Expression() {
        override val children get() = expressions
    }

    data class TypeExpression(
        override val type: String,
        override val metadata: MetadataStore = emptyStore(),
    ) : Expression(), Leaf, TypedDecl

    data class IfExpr(
        val condition: Expression,
        val body: Expression,
        val elifs: List<ElifExpr>,
        val el: ElseExpr?,
        override val metadata: MetadataStore = emptyStore(),
    ) : CodeFlowExpr() {
        override val children get() = listOf(condition, body) + elifs + listOfNotNull(el)

        data

        class ElifExpr(
            val condition: Expression,
            val body: Expression,
            override val metadata: MetadataStore = emptyStore(),
        ) : Node {
            override val children get() = listOf(condition, body)
        }

        data class ElseExpr(
            val body: Expression,
            override val metadata: MetadataStore = emptyStore(),
        ) : Node {
            override val children get() = listOf(body)
        }
    }

    data class WhileExpr(
        val condition: Expression,
        val body: Expression,
        override val metadata: MetadataStore = emptyStore(),
    ) : CodeFlowExpr() {
        override val children get() = listOf(condition, body)
    }
}
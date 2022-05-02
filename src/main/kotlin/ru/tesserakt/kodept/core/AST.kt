package ru.tesserakt.kodept.core

import arrow.core.NonEmptyList
import ru.tesserakt.kodept.lexer.CodePoint
import java.math.BigDecimal
import java.math.BigInteger

data class AST(val root: Node, val fileName: String) {
    sealed interface Node {
        val children: List<Node>
        val coordinates: CodePoint
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
        sealed class Simple : Term(), NamedDecl
    }

    sealed class CodeFlowExpr : Expression()

    data class FileDecl(
        val modules: NonEmptyList<ModuleDecl>,
        override val metadata: MetadataStore = emptyStore(),
    ) : Node {
        override val coordinates: CodePoint = CodePoint(0, 0)
        override val children get() = modules
    }

    data class ModuleDecl(
        override val name: String,
        val global: Boolean,
        val rest: List<TopLevelDecl>,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : NamedDecl {
        override val children get() = rest
    }

    data class StructDecl(
        override val name: String,
        val alloc: List<Parameter>,
        val rest: List<ObjectLevelDecl>,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : ObjectDecl(), TopLevelDecl, NamedDecl {
        override val children get() = alloc + rest

        data class Parameter(
            override val name: String,
            val type: TypeExpression,
            override val coordinates: CodePoint,
            override val metadata: MetadataStore = emptyStore(),
        ) : Leaf, NamedDecl
    }

    data class EnumDecl(
        override val name: String,
        val stackBased: Boolean,
        val enumEntries: List<Entry>,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : ObjectDecl(), TopLevelDecl, NamedDecl {
        override val children get() = enumEntries

        data class Entry(
            override val name: String,
            override val coordinates: CodePoint,
            override val metadata: MetadataStore = emptyStore(),
        ) : ObjectDecl(), Leaf, NamedDecl
    }

    data class TraitDecl(
        override val name: String,
        val rest: List<ObjectLevelDecl>,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : ObjectDecl(), TopLevelDecl, NamedDecl {
        override val children get() = rest
    }

    data class FunctionDecl(
        override val name: String,
        val params: List<Parameter>,
        val returns: TypeExpression?,
        val rest: Expression,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : CallableDecl(), TopLevelDecl, NamedDecl, ObjectLevelDecl, BlockLevelDecl {
        override val children get() = params + listOf(rest) + listOfNotNull(returns)

        data class Parameter(
            override val name: String,
            val type: TypeExpression,
            override val coordinates: CodePoint,
            override val metadata: MetadataStore = emptyStore(),
        ) : NamedDecl {
            override val children get() = listOf(type)
        }
    }

    data class VariableDecl(
        override val name: String,
        val mutable: Boolean,
        val type: TypeExpression?,
        override val coordinates: CodePoint,
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
        override val coordinates: CodePoint = decl.coordinates
        override val children get() = listOf(decl, expr)
    }

    data class DecimalLiteral(
        val value: BigInteger,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal.Number(), Leaf

    data class BinaryLiteral(
        val value: BigInteger,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal.Number(), Leaf

    data class OctalLiteral(
        val value: BigInteger,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal.Number(), Leaf

    data class HexLiteral(
        val value: BigInteger,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal.Number(), Leaf

    data class CharLiteral(
        val value: Char,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal(), Leaf

    data class StringLiteral(
        val value: String,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal(), Leaf

    data class FloatingLiteral(
        val value: BigDecimal,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Literal.Number(), Leaf

    data class Mathematical(
        override val left: Expression,
        override val right: Expression,
        val kind: Kind,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary() {
        enum class Kind { Add, Sub, Mul, Div, Mod, Pow }
    }

    data class Logical(
        override val left: Expression,
        override val right: Expression,
        val kind: Kind,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary() {
        enum class Kind { Conjunction, Disjunction }
    }

    data class Comparison(
        override val left: Expression,
        override val right: Expression,
        val kind: Kind,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary() {
        enum class Kind { Less, LessEqual, Equal, NonEqual, GreaterEqual, Greater, Complex }
    }

    data class Binary(
        override val left: Expression,
        override val right: Expression,
        val kind: Kind,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary() {
        enum class Kind { And, Or, Xor }
    }

    data class Negation(
        override val expr: Expression,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Unary()

    data class Inversion(
        override val expr: Expression,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Unary()

    data class BitInversion(
        override val expr: Expression,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Unary()

    data class Absolution(
        override val expr: Expression,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Unary()

    data class Elvis(
        override val left: Expression,
        override val right: Expression,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary()

    data class Assignment(
        override val left: Term,
        override val right: Expression,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Operation.Binary()

    data class Reference(
        override val name: String,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Term.Simple(), Leaf

    data class ResolutionContext(val fromRoot: Boolean, val chain: NonEmptyList<TypeReference>)

    data class TypeReference(
        val type: TypeExpression,
        val resolutionContext: ResolutionContext? = null,
    ) : Term.Simple(), Leaf {
        override val name: String = type.type
        override val coordinates: CodePoint = type.coordinates
        override val metadata: MetadataStore = type.metadata
    }

    data class FunctionCall(
        val reference: Reference,
        val params: List<Expression>,
        val resolutionContext: ResolutionContext? = null,
        override val metadata: MetadataStore = emptyStore(),
    ) : Term.Simple(), NamedDecl by reference {
        override val coordinates: CodePoint = reference.coordinates
        override val children get() = params
    }

    data class TermChain(
        val terms: NonEmptyList<Simple>,
        override val metadata: MetadataStore = emptyStore(),
    ) : Term() {
        override val coordinates: CodePoint = terms.head.coordinates
        override val children get() = terms
    }

    data class ExpressionList(
        val expressions: List<BlockLevelDecl>,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Expression() {
        override val children get() = expressions
    }

    data class TypeExpression(
        override val type: String,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : Expression(), Leaf, TypedDecl

    data class IfExpr(
        val condition: Expression,
        val body: Expression,
        val elifs: List<ElifExpr>,
        val el: ElseExpr?,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : CodeFlowExpr() {
        override val children get() = listOf(condition, body) + elifs + listOfNotNull(el)

        data

        class ElifExpr(
            val condition: Expression,
            val body: Expression,
            override val coordinates: CodePoint,
            override val metadata: MetadataStore = emptyStore(),
        ) : Node {
            override val children get() = listOf(condition, body)
        }

        data class ElseExpr(
            val body: Expression,
            override val coordinates: CodePoint,
            override val metadata: MetadataStore = emptyStore(),
        ) : Node {
            override val children get() = listOf(body)
        }
    }

    data class WhileExpr(
        val condition: Expression,
        val body: Expression,
        override val coordinates: CodePoint,
        override val metadata: MetadataStore = emptyStore(),
    ) : CodeFlowExpr() {
        override val children get() = listOf(condition, body)
    }
}
package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import arrow.core.nonEmptyListOf
import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken
import ru.tesserakt.kodept.visitor.Acceptable
import ru.tesserakt.kodept.visitor.NodeVisitor
import java.math.BigDecimal
import java.math.BigInteger

data class AST(val root: Node) {
    sealed interface Node : Acceptable {
        val children: List<Node>

        override fun <T> accept(visitor: NodeVisitor<T>) = when (this) {
            is IfExpr -> visitor.visit(this)
            is WhileExpr -> visitor.visit(this)
            is ExpressionList -> visitor.visit(this)
            is CharLiteral -> visitor.visit(this)
            is BinaryLiteral -> visitor.visit(this)
            is DecimalLiteral -> visitor.visit(this)
            is FloatingLiteral -> visitor.visit(this)
            is HexLiteral -> visitor.visit(this)
            is OctalLiteral -> visitor.visit(this)
            is StringLiteral -> visitor.visit(this)
            is Assignment -> visitor.visit(this)
            is Binary -> visitor.visit(this)
            is Comparison -> visitor.visit(this)
            is Elvis -> visitor.visit(this)
            is Logical -> visitor.visit(this)
            is Mathematical -> visitor.visit(this)
            is Absolution -> visitor.visit(this)
            is BitInversion -> visitor.visit(this)
            is Inversion -> visitor.visit(this)
            is Negation -> visitor.visit(this)
            is TermChain -> visitor.visit(this)
            is UnresolvedFunctionCall -> visitor.visit(this)
            is UnresolvedReference -> visitor.visit(this)
            is TypeExpression -> visitor.visit(this)
            is FunctionDecl -> visitor.visit(this)
            is InitializedVar -> visitor.visit(this)
            is VariableDecl -> visitor.visit(this)
            is FileDecl -> visitor.visit(this)
            is EnumDecl.Entry -> visitor.visit(this)
            is EnumDecl -> visitor.visit(this)
            is ModuleDecl -> visitor.visit(this)
            is StructDecl.Parameter -> visitor.visit(this)
            is FunctionDecl.Parameter -> visitor.visit(this)
            is StructDecl -> visitor.visit(this)
            is TraitDecl -> visitor.visit(this)
            is IfExpr.ElifExpr -> visitor.visit(this)
            is IfExpr.ElseExpr -> visitor.visit(this)
        }

        override fun <T> acceptRecursively(visitor: NodeVisitor<T>): NonEmptyList<T> =
            nonEmptyListOf(accept(visitor)) + children.flatMap { it.acceptRecursively(visitor) }
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

    sealed class Term : Expression()

    sealed class CodeFlowExpr : Expression()

    data class FileDecl(val modules: List<ModuleDecl>) : Node {
        override val children: List<Node> = modules
    }

    data class ModuleDecl(override val name: String, val global: Boolean, val rest: List<TopLevelDecl>) : NamedDecl {
        override val children: List<Node> = rest
    }

    data class StructDecl(override val name: String, val alloc: List<Parameter>, val rest: List<ObjectLevelDecl>) :
        ObjectDecl(), TopLevelDecl,
        NamedDecl {
        override val children: List<Node> = alloc + rest

        data class Parameter(override val name: String, val type: TypeExpression) : Leaf, NamedDecl
    }

    data class EnumDecl(override val name: String, val stackBased: Boolean, val enumEntries: List<Entry>) :
        ObjectDecl(), TopLevelDecl, NamedDecl {
        override val children: List<Node> = enumEntries

        data class Entry(override val name: String) : ObjectDecl(), Leaf, NamedDecl
    }

    data class TraitDecl(override val name: String, val rest: List<ObjectLevelDecl>) : ObjectDecl(), TopLevelDecl,
        NamedDecl {
        override val children: List<Node> = rest
    }

    data class FunctionDecl(
        override val name: String,
        val params: List<Parameter>,
        val returns: TypeExpression?,
        val rest: Expression,
    ) : CallableDecl(), TopLevelDecl, NamedDecl, ObjectLevelDecl, BlockLevelDecl {
        override val children: List<Node> = params + listOf(rest) + listOfNotNull(returns)

        data class Parameter(override val name: String, val type: TypeExpression) : NamedDecl {
            override val children: List<Node> = listOf(type)
        }
    }

    data class VariableDecl(override val name: String, val mutable: Boolean, val type: TypeExpression?) :
        CallableDecl(), NamedDecl, BlockLevelDecl {
        override val children: List<Node> = listOfNotNull(type)
    }

    data class InitializedVar(val decl: VariableDecl, val expr: Expression) : CallableDecl(), NamedDecl by decl,
        BlockLevelDecl by decl {
        override val children: List<Node> = listOf(decl, expr)

        override fun <T> accept(visitor: NodeVisitor<T>) = visitor.visit(this)
        override fun <T> acceptRecursively(visitor: NodeVisitor<T>) =
            nonEmptyListOf(accept(visitor)) + children.flatMap { it.acceptRecursively(visitor) }
    }

    data class DecimalLiteral(val value: BigInteger) : Literal.Number(), Leaf

    data class BinaryLiteral(val value: BigInteger) : Literal.Number(), Leaf

    data class OctalLiteral(val value: BigInteger) : Literal.Number(), Leaf

    data class HexLiteral(val value: BigInteger) : Literal.Number(), Leaf

    data class CharLiteral(val value: Char) : Literal(), Leaf

    data class StringLiteral(val value: String) : Literal(), Leaf

    data class FloatingLiteral(val value: BigDecimal) : Literal.Number(), Leaf

    data class Mathematical(override val left: Expression, override val right: Expression, val kind: Kind) :
        Operation.Binary() {
        enum class Kind { Add, Sub, Mul, Div, Mod, Pow }
    }

    data class Logical(override val left: Expression, override val right: Expression, val kind: Kind) :
        Operation.Binary() {
        enum class Kind { Conjunction, Disjunction }
    }

    data class Comparison(override val left: Expression, override val right: Expression, val kind: Kind) :
        Operation.Binary() {
        enum class Kind { Less, LessEqual, Equal, NonEqual, GreaterEqual, Greater, Complex }
    }

    data class Binary(override val left: Expression, override val right: Expression, val kind: Kind) :
        Operation.Binary() {
        enum class Kind { And, Or, Xor }
    }

    data class Negation(override val expr: Expression) : Operation.Unary()

    data class Inversion(override val expr: Expression) : Operation.Unary()

    data class BitInversion(override val expr: Expression) : Operation.Unary()

    data class Absolution(override val expr: Expression) : Operation.Unary()

    data class Elvis(override val left: Expression, override val right: Expression) : Operation.Binary()

    data class Assignment(override val left: Expression, override val right: Expression) : Operation.Binary()

    data class UnresolvedReference(val name: String) : Term(), Leaf

    data class UnresolvedFunctionCall(val name: UnresolvedReference, val params: List<Expression>) : Term() {
        override val children: List<Node> = params
    }

    data class TermChain(val terms: NonEmptyList<Term>) : Term() {
        override val children: List<Node> = terms
    }

    data class ExpressionList(val expressions: List<BlockLevelDecl>) : Expression() {
        override val children: List<Node> = expressions
    }

    data class TypeExpression(override val type: String) : Expression(), Leaf, TypedDecl {
        companion object {
            val unit = TypeExpression("()")
        }
    }

    data class IfExpr(val condition: Expression, val body: Expression, val elifs: List<ElifExpr>, val el: ElseExpr?) :
        CodeFlowExpr() {
        override val children: List<Node> = listOf(condition, body) + elifs + listOfNotNull(el)

        data class ElifExpr(val condition: Expression, val body: Expression) : Node {
            override val children: List<Node> = listOf(condition, body)
        }

        data class ElseExpr(val body: Expression) : Node {
            override val children: List<Node> = listOf(body)
        }
    }

    data class WhileExpr(val condition: Expression, val body: Expression) : CodeFlowExpr() {
        override val children: List<Node> = listOf(condition, body)
    }
}

inline fun <reified T> trailing(
    other: Parser<T>,
    separator: Parser<*> = ExpressionToken.SEMICOLON or ExpressionToken.NEWLINE,
    atLeast: Int = 0,
) = when (atLeast) {
    0 -> zeroOrMore(other * -separator) * optional(other * -optional(separator)) use {
        t1 + listOfNotNull(t2)
    }
    1 -> (zeroOrMore(other * -separator) * other use { t1 + listOf(t2) }) or (oneOrMore(other * -separator))
    else -> (((atLeast - 1) timesOrMore other * -separator) * other use { t1 + listOf(t2) }) or
            (atLeast timesOrMore (other * -separator))
}

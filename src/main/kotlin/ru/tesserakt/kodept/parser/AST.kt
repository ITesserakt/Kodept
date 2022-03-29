package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken
import java.math.BigDecimal
import java.math.BigInteger

data class AST(val nodes: Sequence<Node>) {
    sealed interface Node

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
        sealed class Binary : Operation()
        sealed class Unary : Operation()
    }

    sealed class Term : Expression()

    data class FileDecl(val modules: List<ModuleDecl>) : Node

    data class ModuleDecl(override val name: String, val global: Boolean, val rest: List<TopLevelDecl>) : NamedDecl

    data class StructDecl(override val name: String, val rest: List<ObjectLevelDecl>) : ObjectDecl(), TopLevelDecl,
        NamedDecl

    data class EnumDecl(override val name: String, val stackBased: Boolean, val enumEntries: List<Entry>) :
        ObjectDecl(), TopLevelDecl, NamedDecl {
        data class Entry(override val name: String) : ObjectDecl(), NamedDecl
    }

    data class TraitDecl(override val name: String, val rest: List<ObjectLevelDecl>) : ObjectDecl(), TopLevelDecl,
        NamedDecl

    data class FunctionDecl(
        override val name: String,
        val params: List<Parameter>,
        val returns: TypeExpression?,
        val rest: List<BlockLevelDecl>
    ) : CallableDecl(), TopLevelDecl, NamedDecl, ObjectLevelDecl, BlockLevelDecl {
        data class Parameter(override val name: String, val type: TypeExpression) : NamedDecl
    }

    data class VariableDecl(override val name: String, val mutable: Boolean, val type: TypeExpression?) :
        CallableDecl(), NamedDecl, BlockLevelDecl

    data class InitializedVar(val decl: VariableDecl, val expr: Expression) : CallableDecl(), NamedDecl by decl,
        BlockLevelDecl by decl

    data class DecimalLiteral(val value: BigInteger) : Literal.Number()

    data class BinaryLiteral(val value: BigInteger) : Literal.Number()

    data class OctalLiteral(val value: BigInteger) : Literal.Number()

    data class HexLiteral(val value: BigInteger) : Literal.Number()

    data class CharLiteral(val value: Char) : Literal()

    data class StringLiteral(val value: String) : Literal()

    data class FloatingLiteral(val value: BigDecimal) : Literal.Number()

    data class Mathematical(val left: Expression, val right: Expression, val kind: Kind) : Operation.Binary() {
        enum class Kind { Add, Sub, Mul, Div, Mod, Pow }
    }

    data class Logical(val left: Expression, val right: Expression, val kind: Kind) : Operation.Binary() {
        enum class Kind { Conjunction, Disjunction }
    }

    data class Comparison(val left: Expression, val right: Expression, val kind: Kind) : Operation.Binary() {
        enum class Kind { Less, LessEqual, Equal, GreaterEqual, Greater, Complex }
    }

    data class Binary(val left: Expression, val right: Expression, val kind: Kind) : Operation.Binary() {
        enum class Kind { And, Or, Xor }
    }

    data class Negation(val expr: Expression) : Operation.Unary()

    data class Inversion(val expr: Expression) : Operation.Unary()

    data class BitInversion(val expr: Expression) : Operation.Unary()

    data class Absolution(val expr: Expression) : Operation.Unary()

    data class Elvis(val left: Expression, val right: Expression) : Operation.Binary()

    data class Assignment(val left: Expression, val right: Expression) : Operation.Binary()

    data class UnresolvedReference(val name: String) : Term()

    data class UnresolvedFunctionCall(val name: UnresolvedReference, val params: List<Expression>) : Term()

    data class TermChain(val terms: List<Term>) : Term()

    data class ExpressionList(val expressions: List<BlockLevelDecl>) : Expression()

    data class TypeExpression(override val type: String) : Expression(), TypedDecl {
        companion object {
            val unit = TypeExpression("()")
        }
    }
}

inline fun <reified T> Grammar<AST.Node>.trailing(
    other: Parser<T>,
    separator: Parser<*> = ExpressionToken.SEMICOLON or ExpressionToken.NEWLINE,
    atLeast: Int = 0
) =
    ((atLeast - 1).coerceAtLeast(0) timesOrMore (other * -separator)) * when (atLeast) {
        0 -> optional(other * -optional(separator))
        else -> other * -optional(separator)
    } map { it.t1 + listOfNotNull(it.t2) }

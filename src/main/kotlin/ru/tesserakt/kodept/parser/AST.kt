package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken
import java.math.BigDecimal
import java.math.BigInteger

data class AST(val root: Node) {
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

    data class StructDecl(override val name: String, val alloc: List<Parameter>, val rest: List<ObjectLevelDecl>) :
        ObjectDecl(), TopLevelDecl,
        NamedDecl {
        data class Parameter(override val name: String, val type: TypeExpression) : NamedDecl
    }

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
        enum class Kind { Less, LessEqual, Equal, NonEqual, GreaterEqual, Greater, Complex }
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

fun AST.drawTree(): String {
    operator fun String.times(n: Int) = List(n) { this }.joinToString("")

    fun AST.Node.drawNode(nesting: Int, ident: (Int) -> String = { " " * it }): String = when (this) {
        is AST.ExpressionList -> expressions.joinToString("\n") { it.drawNode(nesting + 1) }
        is AST.CharLiteral -> value.toString().prependIndent(ident(nesting))
        is AST.BinaryLiteral -> value.toString().prependIndent(ident(nesting))
        is AST.DecimalLiteral -> value.toString().prependIndent(ident(nesting))
        is AST.FloatingLiteral -> value.toString().prependIndent(ident(nesting))
        is AST.HexLiteral -> value.toString().prependIndent(ident(nesting))
        is AST.OctalLiteral -> value.toString().prependIndent(ident(nesting))
        is AST.StringLiteral -> value.prependIndent(ident(nesting))
        is AST.Assignment -> """Assignment:
            |  - left:
            |${left.drawNode(nesting + 1)}
            |  - right:
            |${right.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.Binary -> """$kind:
            |  - left:
            |${left.drawNode(nesting + 1)}
            |  - right:
            |${right.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.Comparison -> """$kind:
            |  - left:
            |${left.drawNode(nesting + 1)}
            |  - right:
            |${right.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.Elvis -> """Elvis:
            |  - left:
            |${left.drawNode(nesting + 1)}
            |  - right:
            |${right.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.Logical -> """$kind:
            |  - left:
            |${left.drawNode(nesting + 1)}
            |  - right:
            |${right.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.Mathematical -> """$kind:
            |  - left:
            |${left.drawNode(nesting + 1)}
            |  - right:
            |${right.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.Absolution -> """Absolution:
            |  - left:
            |${expr.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.BitInversion -> """Bit inversion:
            |  - left:
            |${expr.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.Inversion -> """Inversion:
            |  - left:
            |${expr.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.Negation -> """Negation:
            |  - left:
            |${expr.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.TermChain -> terms.joinToString("\n") { it.drawNode(nesting + 1) }
        is AST.UnresolvedFunctionCall -> """Function reference(${name.name}):
            |  - params:
            |${params.joinToString("\n") { it.drawNode(nesting + 1) }}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.UnresolvedReference -> name.prependIndent(ident(nesting))
        is AST.TypeExpression -> type.prependIndent(ident(nesting))
        is AST.FunctionDecl -> """Function($name) 
            |  - returns:
            |${returns?.drawNode(nesting + 1).orEmpty()}
            |  - params:
            |${params.joinToString("\n") { it.drawNode(nesting + 1) }}
            |  - body:
            |${rest.joinToString("\n") { it.drawNode(nesting + 1) }}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.InitializedVar -> """${if (decl.mutable) "Mutable" else "Immutable"} initialized var($name) 
            |  - type:
            |${decl.type?.drawNode(nesting + 1).orEmpty()}
            |  - expression:
            |${expr.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.VariableDecl -> """${if (mutable) "Mutable" else "Immutable"} var($name) 
            |  - type:
            |${type?.drawNode(nesting + 1)}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.FileDecl -> """File:
            |  - modules:
            |${modules.joinToString("\n") { it.drawNode(nesting + 1) }}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.EnumDecl.Entry -> name.prependIndent(ident(nesting))
        is AST.EnumDecl -> """${if (stackBased) "Stack" else "Heap"} enum($name)
            |  - entries:
            |${enumEntries.joinToString("\n") { it.drawNode(nesting + 1) }}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.ModuleDecl -> """${if (global) "Global m" else "M"}odule($name)
            |  - entries:
            |${rest.joinToString("\n") { it.drawNode(nesting + 1) }}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.StructDecl.Parameter -> """$name: $type""".prependIndent(ident(nesting))
        is AST.FunctionDecl.Parameter -> """$name: $type""".prependIndent(ident(nesting))
        is AST.StructDecl -> """Struct($name)
            |  - params:
            |${alloc.joinToString("\n") { it.drawNode(nesting + 1) }}
            |  - body:
            |${rest.joinToString("\n") { it.drawNode(nesting + 1) }}
        """.trimMargin().prependIndent(ident(nesting))
        is AST.TraitDecl -> """Trait($name)
            |  - body:
            |${rest.joinToString("\n") { it.drawNode(nesting + 1) }}
        """.trimMargin().prependIndent(ident(nesting))
    }

    return root.drawNode(0)
}

inline fun <reified T> Grammar<AST.Node>.trailing(
    other: Parser<T>,
    separator: Parser<*> = ExpressionToken.SEMICOLON or ExpressionToken.NEWLINE,
    atLeast: Int = 0
) = when (atLeast) {
    0 -> zeroOrMore(other * -separator) * optional(other * -optional(separator)) use {
        t1 + listOfNotNull(t2)
    }
    1 -> (zeroOrMore(other * -separator) * other use { t1 + listOf(t2) }) or (oneOrMore(other * -separator))
    else -> (((atLeast - 1) timesOrMore other * -separator) * other use { t1 + listOf(t2) }) or
            (atLeast timesOrMore (other * -separator))
}

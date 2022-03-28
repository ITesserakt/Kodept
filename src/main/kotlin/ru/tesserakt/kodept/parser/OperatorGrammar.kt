package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.grammar.parser
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import com.github.h0tk3y.betterParse.parser.Parser
import com.github.h0tk3y.betterParse.utils.Tuple2
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.parser.AST.*

object OperatorGrammar : Grammar<Expression>() {
    private fun resolveMathOperation(op: TokenMatch) = when (op.type) {
        TIMES.token -> Mathematical.Kind.Mul
        DIV.token -> Mathematical.Kind.Div
        MOD.token -> Mathematical.Kind.Mod
        PLUS.token -> Mathematical.Kind.Add
        SUB.token -> Mathematical.Kind.Sub
        else -> throw IllegalArgumentException("Impossible")
    }

    private fun resolveCmpOperation(op: TokenMatch) = when (op.type) {
        SPACESHIP.token -> Comparison.Kind.Complex
        LESS_EQUALS.token -> Comparison.Kind.LessEqual
        GREATER_EQUALS.token -> Comparison.Kind.GreaterEqual
        EQUIV.token -> Comparison.Kind.Equal
        LESS.token -> Comparison.Kind.Less
        GREATER.token -> Comparison.Kind.Greater
        else -> throw IllegalArgumentException("Impossible")
    }

    private fun resolveBinaryOperation(op: TokenMatch) = when (op.type) {
        AND_BIT.token -> Binary.Kind.And
        XOR_BIT.token -> Binary.Kind.Xor
        OR_BIT.token -> Binary.Kind.Or
        else -> throw IllegalArgumentException("Impossible")
    }

    private fun resolveLogicalOperation(op: TokenMatch) = when (op.type) {
        AND_LOGIC.token -> Logical.Kind.Conjunction
        OR_LOGIC.token -> Logical.Kind.Disjunction
        else -> throw IllegalArgumentException("Impossible")
    }

    private infix fun <E, A : E, B : E, R : E> Parser<Tuple2<A, List<Tuple2<TokenMatch, B>>>>.leftFold(construct: (E, TokenMatch, B) -> R) =
        map { (a, tail) ->
            if (tail.isEmpty())
                a
            else {
                val (head, rest) = tail.first() to tail.drop(1)
                val (op, b) = head
                rest.fold(construct(a, op, b)) { acc, (ops, bs) ->
                    construct(acc, ops, bs)
                }
            }
        }

    val atom by (-LPAREN * this * -RPAREN) or ExpressionGrammar

    val topExpr: Parser<Expression> by (-SUB * parser { topExpr } use (::Negation)) or
            (-NOT_LOGIC * parser { topExpr } use (::Inversion)) or
            (-NOT_BIT * parser { topExpr } use (::BitInversion)) or
            (-PLUS * parser { topExpr } use (::Absolution)) or
            atom

    val mulExpr by topExpr * zeroOrMore((TIMES or DIV or MOD) * topExpr) leftFold { a, op, b ->
        Mathematical(a, b, resolveMathOperation(op))
    }

    val addExpr by mulExpr * zeroOrMore((PLUS or SUB) * mulExpr) leftFold { a, op, b ->
        Mathematical(a, b, resolveMathOperation(op))
    }

    val complexCmpExpr = addExpr * zeroOrMore(SPACESHIP * addExpr) leftFold { a, op, b ->
        Comparison(a, b, resolveCmpOperation(op))
    }

    val compoundCmpExpr by complexCmpExpr * zeroOrMore((LESS_EQUALS or EQUIV or GREATER_EQUALS) * complexCmpExpr) leftFold { a, op, b ->
        Comparison(a, b, resolveCmpOperation(op))
    }

    val cmpExpr by compoundCmpExpr * zeroOrMore((LESS or GREATER) * compoundCmpExpr) leftFold { a, op, b ->
        Comparison(a, b, resolveCmpOperation(op))
    }

    val bitExpr by cmpExpr * zeroOrMore((AND_BIT or XOR_BIT or OR_BIT) * cmpExpr) leftFold { a, op, b ->
        Binary(a, b, resolveBinaryOperation(op))
    }

    val logicExpr by bitExpr * zeroOrMore((AND_LOGIC or OR_LOGIC) * bitExpr) leftFold { a, op, b ->
        Logical(a, b, resolveLogicalOperation(op))
    }

    val elvis: Parser<Expression> by logicExpr * optional(ELVIS * parser { elvis }) map { (a, rest) ->
        when (rest) {
            null -> a
            else -> Elvis(a, rest.t2)
        }
    }

    override val rootParser by elvis
}
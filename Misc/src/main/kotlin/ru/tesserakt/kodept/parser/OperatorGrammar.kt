package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import arrow.core.None
import arrow.core.Some
import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.grammar.parser
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import com.github.h0tk3y.betterParse.parser.Parser
import com.github.h0tk3y.betterParse.utils.Tuple2
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.symbol
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object OperatorGrammar : Grammar<RLT.ExpressionNode>() {
    private infix fun <E, A : E, B : E, R : E> Parser<Tuple2<A, List<Tuple2<TokenMatch, B>>>>.leftFold(construct: (E, RLT.Symbol, B) -> R) =
        map { (a, tail) ->
            when (val rest = NonEmptyList.fromList(tail)) {
                None -> a
                is Some -> {
                    val (op, b) = rest.value.head
                    rest.value.tail.fold(construct(a, op.symbol(), b)) { acc, (ops, bs) ->
                        construct(acc, ops.symbol(), bs)
                    }
                }
            }
        }

    private infix fun <E, A : E, B : E, R : E> Parser<Tuple2<A, Tuple2<TokenMatch, B>?>>.rightFold(construct: (E, RLT.Symbol, B) -> R) =
        map { (a, tail) ->
            when (tail) {
                null -> a
                else -> construct(a, tail.t1.symbol(), tail.t2)
            }
        }

    val atom by (-LPAREN * this * -RPAREN) or ExpressionGrammar
    val access by atom * zeroOrMore(DOT * atom) leftFold RLT::BinaryOperation
    val topExpr: Parser<RLT.ExpressionNode> by (SUB or NOT_LOGIC or NOT_BIT or PLUS) * parser { topExpr } map {
        RLT.UnaryOperation(it.t2, it.t1.symbol())
    } or access

    val powExpr: Parser<RLT.ExpressionNode> by topExpr * optional(POW * parser { powExpr }) rightFold RLT::BinaryOperation
    val mulExpr by powExpr * zeroOrMore((TIMES or DIV or MOD) * powExpr) leftFold RLT::BinaryOperation
    val addExpr by mulExpr * zeroOrMore((PLUS or SUB) * mulExpr) leftFold RLT::BinaryOperation
    val complexCmpExpr = addExpr * zeroOrMore(SPACESHIP * addExpr) leftFold RLT::BinaryOperation
    val compoundCmpExpr by complexCmpExpr * zeroOrMore((LESS_EQUALS or EQUIV or NOT_EQUIV or GREATER_EQUALS) * complexCmpExpr) leftFold RLT::BinaryOperation
    val cmpExpr by compoundCmpExpr * zeroOrMore((LESS or GREATER) * compoundCmpExpr) leftFold RLT::BinaryOperation
    val bitExpr by cmpExpr * zeroOrMore((AND_BIT or XOR_BIT or OR_BIT) * cmpExpr) leftFold RLT::BinaryOperation
    val logicExpr by bitExpr * zeroOrMore((AND_LOGIC or OR_LOGIC) * bitExpr) leftFold RLT::BinaryOperation
//    val elvis: Parser<RLT.ExpressionNode> by logicExpr * optional(ELVIS * parser { elvis }) rightFold RLT::BinaryOperation

    override val rootParser by logicExpr
}
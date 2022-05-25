package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.grammar.parser
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object TypeGrammar : Grammar<RLT.TypeNode>() {
    val type = TYPE map { RLT.UserSymbol.Type(it) }

    val tuple = LPAREN * trailing(parser { rootParser }, COMMA) * RPAREN map {
        RLT.TupleType(RLT.Symbol(it.t1), it.t2, RLT.Symbol(it.t3))
    }

    val union = LPAREN * trailing(parser { rootParser }, OR_BIT, atLeast = 2) * RPAREN map {
        RLT.UnionType(RLT.Symbol(it.t1), NonEmptyList.fromListUnsafe(it.t2), RLT.Symbol(it.t3))
    }

    override val rootParser: Parser<RLT.TypeNode> by type or union or tuple

    val coloned by COLON * rootParser
}
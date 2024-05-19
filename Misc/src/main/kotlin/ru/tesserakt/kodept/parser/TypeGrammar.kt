package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.grammar.parser
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.symbol
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object TypeGrammar : Grammar<RLT.TypeNode>() {
    val type = TermGrammar.contextualType or TermGrammar.typeReference

    val tuple = LPAREN and strictTrailing(parser { rootParser }, COMMA) * RPAREN map {
        RLT.TupleType(it.t1.symbol(), it.t2, it.t3.symbol())
    }

    val union = LPAREN and strictTrailing(parser { rootParser }, OR_BIT, atLeast = 2) * RPAREN map {
        RLT.UnionType(it.t1.symbol(), NonEmptyList.fromListUnsafe(it.t2), it.t3.symbol())
    }

    override val rootParser: Parser<RLT.TypeNode> by type or union or tuple

    val coloned by COLON * rootParser
}
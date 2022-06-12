package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.symbol
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object ExpressionGrammar : Grammar<RLT.ExpressionNode>() {
    val application by (TermGrammar.contextualReference or TermGrammar.reference) * oneOrMore(
        -!NEWLINE * LPAREN and strictTrailing(OperatorGrammar, COMMA) * RPAREN
    ) map { (head, tail) ->
        RLT.Application(head, tail.map { (lp, p, rp) ->
            RLT.ParameterTuple(lp.symbol(), p.map(RLT::Parameter), rp.symbol())
        })
    }

    override val rootParser by application or LiteralGrammar or TermGrammar or CodeFlowGrammar
}
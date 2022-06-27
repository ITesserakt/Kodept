package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.symbol
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object ExpressionGrammar : Grammar<RLT.ExpressionNode>() {
    val lambda by -LAMBDA and strictTrailing(
        TermGrammar.reference,
        COMMA
    ) * FLOW * OperatorGrammar map { (it, expr) ->
        val (params, flow) = it
        RLT.Lambda(params, flow.symbol(), expr)
    }

    override val rootParser by
    lambda or LiteralGrammar or TermGrammar or CodeFlowGrammar
}
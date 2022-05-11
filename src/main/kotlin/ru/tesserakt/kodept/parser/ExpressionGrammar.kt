package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.grammar.Grammar

object ExpressionGrammar : Grammar<RLT.ExpressionNode>() {
    override val rootParser by LiteralGrammar or TermGrammar or CodeFlowGrammar
}
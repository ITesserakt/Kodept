package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.optional
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.keyword
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object MatchGrammar : Grammar<RLT.Match>() {
    val branch by OperatorGrammar * BlockLevelGrammar.simple map { (condition, body) ->
        RLT.Match.Branch(condition, body)
    }

    override val rootParser by MATCH * optional(OperatorGrammar) * LBRACE and strictTrailing(
        branch, atLeast = 1, separator = COMMA
    ) * RBRACE map { (matchToken, variable, _, branches, _) ->
        RLT.Match(matchToken.keyword(), variable, NonEmptyList.fromListUnsafe(branches))
    }
}
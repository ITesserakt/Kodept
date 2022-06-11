package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import arrow.core.nonEmptyListOf
import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.combinators.use
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object FileGrammar : Grammar<RLT.File>() {
    val moduleStatement by (MODULE * TYPE * LBRACE) and strictTrailing(TopLevelGrammar) * RBRACE map { (moduleToken, name, lb, rest, rb) ->
        RLT.Module.Ordinary(RLT.Keyword(moduleToken), RLT.UserSymbol.Type(name), RLT.Symbol(lb), rest, RLT.Symbol(rb))
    }

    val globalModuleStatement by MODULE * TYPE * FLOW * trailingUntilEOF(TopLevelGrammar) map { (moduleToken, name, f, rest) ->
        RLT.Module.Global(RLT.Keyword(moduleToken), RLT.UserSymbol.Type(name), RLT.Symbol(f), rest)
    }

    override val rootParser: Parser<RLT.File> by
    (trailingUntilEOF(moduleStatement) map { NonEmptyList.fromListUnsafe(it) } use RLT::File) or
            (globalModuleStatement map { RLT.File(nonEmptyListOf(it)) })
}
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
import ru.tesserakt.kodept.core.keyword
import ru.tesserakt.kodept.core.symbol
import ru.tesserakt.kodept.core.type
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object FileGrammar : Grammar<RLT.File>() {
    val moduleStatement by (MODULE * TYPE * LBRACE) and strictTrailing(TopLevelGrammar) * RBRACE map { (moduleToken, name, lb, rest, rb) ->
        RLT.Module.Ordinary(moduleToken.keyword(), name.type(), lb.symbol(), rest, rb.symbol())
    }

    val globalModuleStatement by MODULE * TYPE * FLOW * trailingUntilEOF(TopLevelGrammar) map { (moduleToken, name, f, rest) ->
        RLT.Module.Global(moduleToken.keyword(), name.type(), f.symbol(), rest)
    }

    override val rootParser: Parser<RLT.File> by
    (trailingUntilEOF(moduleStatement, atLeast = 1) map { NonEmptyList.fromListUnsafe(it) } use RLT::File) or
            (globalModuleStatement map { RLT.File(nonEmptyListOf(it)) })
}
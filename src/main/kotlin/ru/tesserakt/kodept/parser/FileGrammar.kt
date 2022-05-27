package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import arrow.core.nonEmptyListOf
import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.parser.RLT.File

object FileGrammar : Grammar<File>() {
    val moduleStatement by (MODULE * TYPE * LBRACE) and strictTrailing(TopLevelGrammar) * RBRACE map { (moduleToken, name, lb, rest, rb) ->
        RLT.Module.Ordinary(RLT.Keyword(moduleToken), RLT.UserSymbol.Type(name), RLT.Symbol(lb), rest, RLT.Symbol(rb))
    }

    val globalModuleStatement by MODULE * TYPE * FLOW * trailingUntilEOF(TopLevelGrammar) map { (moduleToken, name, f, rest) ->
        RLT.Module.Global(RLT.Keyword(moduleToken), RLT.UserSymbol.Type(name), RLT.Symbol(f), rest)
    }

    override val rootParser: Parser<File> by
    (oneOrMore(moduleStatement) map { NonEmptyList.fromListUnsafe(it) } use ::File) or
            (globalModuleStatement map { File(nonEmptyListOf(it)) })
}
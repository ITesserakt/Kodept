package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import arrow.core.nonEmptyListOf
import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.AST.FileDecl
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.toCodePoint

object FileGrammar : Grammar<FileDecl>() {
    val moduleStatement by MODULE * TYPE * -LBRACE * zeroOrMore(TopLevelGrammar) * -RBRACE map { (moduleToken, name, rest) ->
        AST.ModuleDecl(name.text, false, rest, moduleToken.toCodePoint())
    }

    val globalModuleStatement by MODULE * TYPE * -FLOW * zeroOrMore(TopLevelGrammar) map { (moduleToken, name, rest) ->
        AST.ModuleDecl(name.text, true, rest, moduleToken.toCodePoint())
    }

    override val rootParser: Parser<FileDecl> by
    (oneOrMore(moduleStatement) map { NonEmptyList.fromListUnsafe(it) } use ::FileDecl) or
            (globalModuleStatement map { FileDecl(nonEmptyListOf(it)) })
}
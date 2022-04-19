package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.AST
import ru.tesserakt.kodept.AST.FileDecl
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.toCodePoint

object FileGrammar : Grammar<FileDecl>() {
    val moduleStatement by MODULE * IDENTIFIER * -LBRACE * zeroOrMore(TopLevelGrammar) * -RBRACE map { (moduleToken, name, rest) ->
        AST.ModuleDecl(name.text, false, rest, moduleToken.toCodePoint())
    }

    val globalModuleStatement by MODULE * IDENTIFIER * -FLOW * zeroOrMore(TopLevelGrammar) map { (moduleToken, name, rest) ->
        AST.ModuleDecl(name.text, true, rest, moduleToken.toCodePoint())
    }

    override val rootParser: Parser<FileDecl> by
    (oneOrMore(moduleStatement) use ::FileDecl) or (globalModuleStatement map { FileDecl(listOf(it)) })
}
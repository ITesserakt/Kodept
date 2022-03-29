package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.parser.AST.FileDecl

object FileGrammar : Grammar<FileDecl>() {
    val moduleStatement by -MODULE * IDENTIFIER * -LBRACE * zeroOrMore(TopLevelGrammar) * -RBRACE map { (name, rest) ->
        AST.ModuleDecl(name.text, false, rest)
    }

    val globalModuleStatement by -MODULE * IDENTIFIER * -FLOW * zeroOrMore(TopLevelGrammar) map { (name, rest) ->
        AST.ModuleDecl(name.text, true, rest)
    }

    override val rootParser: Parser<FileDecl> by
    (oneOrMore(moduleStatement) use (::FileDecl)) or (globalModuleStatement map { FileDecl(listOf(it)) })
}
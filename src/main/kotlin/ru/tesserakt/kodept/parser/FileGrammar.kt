package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.lexer.Token
import com.github.h0tk3y.betterParse.lexer.Tokenizer
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.Lexer
import ru.tesserakt.kodept.parser.AST.FileDecl

object FileGrammar : Grammar<FileDecl>() {
    override val tokenizer: Tokenizer = Lexer()
    override val tokens: List<Token> = tokenizer.tokens

    private val topLevelStatement by TopLevelGrammar

    val moduleStatement by -MODULE * IDENTIFIER * -LBRACE * zeroOrMore(topLevelStatement) * -RBRACE map { (name, rest) ->
        AST.ModuleDecl(name.text, false, rest)
    }

    val globalModuleStatement by -MODULE * IDENTIFIER * -FLOW * zeroOrMore(topLevelStatement) map { (name, rest) ->
        AST.ModuleDecl(name.text, true, rest)
    }

    override val rootParser: Parser<FileDecl> by
    (oneOrMore(moduleStatement) use (::FileDecl)) or (globalModuleStatement map { FileDecl(listOf(it)) })
}
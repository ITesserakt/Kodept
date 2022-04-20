package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.core.AST

object ObjectLevelGrammar : Grammar<AST.ObjectLevelDecl>() {
    override val rootParser: Parser<AST.ObjectLevelDecl> by FunctionGrammar
}
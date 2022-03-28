package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object BlockLevelGrammar : Grammar<AST.BlockLevelDecl>() {
    private val expression by ExpressionGrammar
    private val functionStatement by FunctionGrammar

    val variableStatement by (VAL or VAR) * IDENTIFIER * optional(-COLON * (TYPE or TYPE_GAP)) * expression map { (mutable, name, type, expr) ->
        AST.VariableDecl(name.text, mutable.type == VAR.token, expr)
    }

    override val rootParser: Parser<AST.BlockLevelDecl> by functionStatement or variableStatement or expression
}
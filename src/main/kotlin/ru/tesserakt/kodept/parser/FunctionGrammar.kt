package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object FunctionGrammar : Grammar<AST.FunctionDecl>() {
    private val blockLevelStatement by BlockLevelGrammar

    val typed = IDENTIFIER * -COLON * TYPE map { (name, type) -> AST.FunctionDecl.Parameter(name.text, type.text) }

    private val parameterList by -LPAREN * (zeroOrMore(typed * -COMMA) * optional(typed * -optional(COMMA)) map {
        it.t1 + listOfNotNull(it.t2)
    }) * -RPAREN

    override val rootParser by -FUN * IDENTIFIER * parameterList * optional(-COLON * TYPE) * -LBRACE * statements(
        blockLevelStatement
    ) * -RBRACE map { (name, params, returnType, rest) ->
        AST.FunctionDecl(
            name.text,
            params,
            returnType?.text?.let { AST.FunctionDecl.ReturnType(it) } ?: AST.FunctionDecl.ReturnType.unit,
            rest
        )
    }
}
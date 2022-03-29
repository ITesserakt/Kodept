package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.combinators.unaryMinus
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object FunctionGrammar : Grammar<AST.FunctionDecl>() {
    private val blockLevelStatement by BlockLevelGrammar

    val typed = IDENTIFIER * TypeGrammar.strict map { (name, type) -> AST.FunctionDecl.Parameter(name.text, type) }

    private val parameterList by -LPAREN * trailing(typed, COMMA) * -RPAREN

    override val rootParser by -FUN * IDENTIFIER * parameterList * TypeGrammar.optional * -LBRACE * trailing(
        blockLevelStatement
    ) * -RBRACE map { (name, params, returnType, rest) ->
        AST.FunctionDecl(
            name.text,
            params,
            returnType,
            rest
        )
    }
}
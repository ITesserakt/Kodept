package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.combinators.unaryMinus
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.toCodePoint

object FunctionGrammar : Grammar<AST.FunctionDecl>() {
    val typed = IDENTIFIER * TypeGrammar.strict map { (name, type) ->
        AST.FunctionDecl.Parameter(name.text, type, name.toCodePoint())
    }

    private val parameterList by -LPAREN * trailing(typed, COMMA) * -RPAREN

    override val rootParser by FUN * IDENTIFIER * parameterList * TypeGrammar.optional * BlockLevelGrammar.body map { (funToken, name, params, returnType, rest) ->
        AST.FunctionDecl(
            name.text,
            params,
            returnType,
            rest,
            funToken.toCodePoint()
        )
    }
}
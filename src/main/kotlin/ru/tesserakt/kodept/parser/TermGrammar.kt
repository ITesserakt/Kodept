package ru.tesserakt.kodept.parser

import arrow.core.curry
import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.combinators.use
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object TermGrammar : Grammar<RLT.TermNode>() {
    val variableReference by IDENTIFIER use { RLT.Reference(RLT.UserSymbol.Identifier(this)) }

    val typeReference by TYPE use { RLT.Reference(RLT.UserSymbol.Type(this)) }

    val reference by variableReference or typeReference

    val contextual by (DOUBLE_COLON * trailing(typeReference, DOUBLE_COLON) map { (global, rest) ->
        rest.fold(RLT.Context.Global(RLT.Symbol(global)) as RLT.Context) { acc, type ->
            RLT.Context.Inner(type, acc)
        }
    }) or (trailing(typeReference, DOUBLE_COLON, atLeast = 1) map {
        it.fold(RLT.Context.Local as RLT.Context) { acc, next ->
            RLT.Context.Inner(next, acc)
        }
    })


    override val rootParser = contextual * reference map (RLT::ContextualReference.curry()) or
            reference
}
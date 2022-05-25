package ru.tesserakt.kodept.parser

import arrow.core.curry
import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object TermGrammar : Grammar<RLT.TermNode>() {
    val variableReference by IDENTIFIER use { RLT.Reference(RLT.UserSymbol.Identifier(this)) }

    val typeReference by TYPE use { RLT.Reference(RLT.UserSymbol.Type(this)) }

    val reference by variableReference or typeReference

    val contextual by optional(DOUBLE_COLON) * oneOrMore(typeReference * DOUBLE_COLON) map { (global, rest) ->
        rest.fold(if (global != null) RLT.Context.Global(RLT.Symbol(global)) else RLT.Context.Local) { acc, next ->
            RLT.Context.Inner(next.t1, acc)
        }
    }


    override val rootParser = contextual * reference map (RLT::ContextualReference.curry()) or
            reference
}
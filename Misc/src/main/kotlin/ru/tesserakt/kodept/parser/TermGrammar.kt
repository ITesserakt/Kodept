package ru.tesserakt.kodept.parser

import arrow.core.curryPair
import arrow.core.flip
import arrow.core.prependTo
import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object TermGrammar : Grammar<RLT.TermNode>() {
    val variableReference by IDENTIFIER use { RLT.Reference(RLT.UserSymbol.Identifier(this)) }

    val typeReference by TYPE use { RLT.Reference(RLT.UserSymbol.Type(this)) }

    val reference by typeReference or variableReference

    // ::X[::]{X[::]}
    private val cc_t_cc_ by DOUBLE_COLON and trailing(
        typeReference, DOUBLE_COLON, atLeast = 1
    ) map { (global, context) ->
        val start: RLT.Context = RLT.Context.Global(RLT.Symbol(global))
        context.dropLast(1).fold(start, RLT.Context::Inner.flip()) to context.last()
    }

    // X::{X::}x
    private val t_cc__v by strictTrailing(
        typeReference,
        DOUBLE_COLON,
        atLeast = 1
    ) * variableReference map { (context, ref) ->
        val start: RLT.Context = RLT.Context.Local
        context.fold(start, RLT.Context::Inner.flip()) to ref
    }

    // X::X{::X}
    private val t_cc_t_ by typeReference * oneOrMore(-DOUBLE_COLON * typeReference) map { (first, rest) ->
        val last = rest.last()
        val start: RLT.Context = RLT.Context.Local
        first.prependTo(rest.dropLast(1)).fold(start, RLT.Context::Inner.flip()) to last
    }

    // ::{X::}x
    private val cc_t_cc__v by DOUBLE_COLON and strictTrailing(
        typeReference,
        DOUBLE_COLON
    ) * variableReference map { (global, context, ref) ->
        val start: RLT.Context = RLT.Context.Global(RLT.Symbol(global))
        context.fold(start, RLT.Context::Inner.flip()) to ref
    }

    val contextual by (t_cc__v or t_cc_t_ or cc_t_cc__v or cc_t_cc_) map (RLT::ContextualReference.curryPair())

    val contextualType by (cc_t_cc_ or t_cc_t_) map (RLT::ContextualReference.curryPair())
    val contextualReference by (t_cc__v or cc_t_cc__v) map (RLT::ContextualReference.curryPair())

    override val rootParser = contextual or reference
}
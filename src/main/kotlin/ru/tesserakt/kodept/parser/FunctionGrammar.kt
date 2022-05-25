package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.utils.Tuple2
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object FunctionGrammar : Grammar<RLT.Function.Bodied>() {
    val strictlyTyped by IDENTIFIER * -COLON * TypeGrammar map { (name, type) ->
        RLT.TypedParameter(RLT.UserSymbol.Identifier(name), type)
    }

    val nonStrictlyTyped by IDENTIFIER * optional(-COLON * TypeGrammar) map { (name, type) ->
        RLT.MaybeTypedParameter(RLT.UserSymbol.Identifier(name), type)
    }

    val strictParameterList by LPAREN * trailing(strictlyTyped, COMMA) * RPAREN map {
        RLT.TypedParameterTuple(RLT.Symbol(it.t1), it.t2, RLT.Symbol(it.t3))
    }

    val parameterList by LPAREN * trailing(nonStrictlyTyped, COMMA) * RPAREN map {
        RLT.MaybeTypedParameterTuple(RLT.Symbol(it.t1), it.t2, RLT.Symbol(it.t3))
    }

    val abstractFunction by FUN * IDENTIFIER * zeroOrMore(strictParameterList) * optional(COLON * TypeGrammar) map { (token, name, tuples, returnType) ->
        val (colon, returnType) = returnType ?: Tuple2(null, null)
        RLT.Function.Abstract(
            token.keyword(),
            RLT.UserSymbol.Identifier(name),
            tuples,
            colon?.let(RLT::Symbol),
            returnType
        )
    }

    val nonStrictParameterFunDecl by FUN * IDENTIFIER * zeroOrMore(parameterList) * optional(COLON * TypeGrammar)

    val function by nonStrictParameterFunDecl * BlockLevelGrammar.body map {
        val (keyword, name, params, returns) = it.t1
        val (colon, returnType) = returns ?: Tuple2(null, null)
        RLT.Function.Bodied(
            keyword.keyword(), RLT.UserSymbol.Identifier(name), params,
            colon?.let(RLT::Symbol), returnType, it.t2
        )
    }

    override val rootParser by function
}
package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.utils.Tuple2
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.keyword
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.ABSTRACT
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.FOREIGN

object FunctionGrammar : Grammar<RLT.Function.Bodied>() {
    val strictlyTyped by IDENTIFIER * -COLON * TypeGrammar map { (name, type) ->
        RLT.TypedParameter(RLT.UserSymbol.Identifier(name), type)
    }

    val nonStrictlyTyped by IDENTIFIER * optional(-COLON * TypeGrammar) map { (name, type) ->
        RLT.MaybeTypedParameter(RLT.UserSymbol.Identifier(name), type)
    }

    val strictParameterList by LPAREN and strictTrailing(strictlyTyped, COMMA) * RPAREN map {
        RLT.TypedParameterTuple(RLT.Symbol(it.t1), it.t2, RLT.Symbol(it.t3))
    }

    val parameterList by LPAREN and strictTrailing(nonStrictlyTyped, COMMA) * RPAREN map {
        RLT.MaybeTypedParameterTuple(RLT.Symbol(it.t1), it.t2, RLT.Symbol(it.t3))
    }

    val abstractFunction by -ABSTRACT * FUN * IDENTIFIER * optionalWithStart(
        LPAREN,
        strictTrailing(strictlyTyped, COMMA) * RPAREN
    ) * zeroOrMore(strictParameterList) * optional(
        COLON * TypeGrammar
    ) map { (token, name, firstTuple, tuples, returnType) ->
        val (colon, returnType) = returnType ?: Tuple2(null, null)
        RLT.Function.Abstract(
            token.keyword(),
            RLT.UserSymbol.Identifier(name),
            listOfNotNull(firstTuple?.let {
                RLT.TypedParameterTuple(
                    RLT.Symbol(it.t1),
                    it.t2.t1,
                    RLT.Symbol(it.t2.t2)
                )
            }) + tuples,
            colon?.let(RLT::Symbol),
            returnType
        )
    }

    val nonStrictParameterFunDecl by FUN * IDENTIFIER * optionalWithStart(
        LPAREN,
        strictTrailing(nonStrictlyTyped, COMMA) * RPAREN
    ) * zeroOrMore(parameterList) * optional(COLON * TypeGrammar)

    val function by nonStrictParameterFunDecl * BlockLevelGrammar.body map { tuple ->
        val (keyword, name, firstParam, params, returns) = tuple.t1
        val (colon, returnType) = returns ?: Tuple2(null, null)
        RLT.Function.Bodied(
            keyword.keyword(),
            RLT.UserSymbol.Identifier(name),
            listOfNotNull(firstParam?.let {
                RLT.MaybeTypedParameterTuple(
                    RLT.Symbol(it.t1),
                    it.t2.t1,
                    RLT.Symbol(it.t2.t2)
                )
            }) + params,
            colon?.let(RLT::Symbol),
            returnType,
            tuple.t2
        )
    }

    val foreignFun by -FOREIGN * FUN * IDENTIFIER * optionalWithStart(
        LPAREN,
        strictTrailing(strictlyTyped, COMMA) * RPAREN
    ) * zeroOrMore(strictParameterList) * optional(
        COLON * TypeGrammar
    ) map { (token, id, first, rest, type) ->
        val (colon, ret) = type ?: Tuple2(null, null)
        RLT.Function.Foreign(token.keyword(), RLT.UserSymbol.Identifier(id), listOfNotNull(first?.let {
            RLT.TypedParameterTuple(RLT.Symbol(it.t1), it.t2.t1, RLT.Symbol(it.t2.t2))
        }) + rest, colon?.let(RLT::Symbol), ret)
    }

    override val rootParser by function
}
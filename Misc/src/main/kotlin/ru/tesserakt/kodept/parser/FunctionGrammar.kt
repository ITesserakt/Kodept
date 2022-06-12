package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.utils.Tuple2
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.identifier
import ru.tesserakt.kodept.core.keyword
import ru.tesserakt.kodept.core.symbol
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.ABSTRACT
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.FOREIGN

object FunctionGrammar : Grammar<RLT.Function.Bodied>() {
    val strictlyTyped by IDENTIFIER * -COLON * TypeGrammar map { (name, type) ->
        RLT.TypedParameter(name.identifier(), type)
    }

    val nonStrictlyTyped by IDENTIFIER * optional(-COLON * TypeGrammar) map { (name, type) ->
        RLT.MaybeTypedParameter(name.identifier(), type)
    }

    val strictParameterList by LPAREN and strictTrailing(strictlyTyped, COMMA) * RPAREN map {
        RLT.TypedParameterTuple(it.t1.symbol(), it.t2, it.t3.symbol())
    }

    val parameterList by LPAREN and strictTrailing(nonStrictlyTyped, COMMA) * RPAREN map {
        RLT.MaybeTypedParameterTuple(it.t1.symbol(), it.t2, it.t3.symbol())
    }

    val abstractFunction by -ABSTRACT * FUN * IDENTIFIER * optionalWithStart(
        LPAREN,
        strictTrailing(strictlyTyped, COMMA) * RPAREN
    ) * zeroOrMore(strictParameterList) * optional(
        COLON * TypeGrammar
    ) map { (token, name, firstTuple, tuples, returnType) ->
        val (colon, returnType) = returnType ?: Tuple2(null, null)
        RLT.Function.Abstract(
            token.keyword(), name.identifier(), listOfNotNull(firstTuple?.let {
                RLT.TypedParameterTuple(
                    it.t1.symbol(),
                    it.t2.t1,
                    it.t2.t2.symbol()
                )
            }) + tuples, colon?.symbol(), returnType
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
            name.identifier(),
            listOfNotNull(firstParam?.let {
                RLT.MaybeTypedParameterTuple(
                    it.t1.symbol(),
                    it.t2.t1,
                    it.t2.t2.symbol()
                )
            }) + params,
            colon?.symbol(),
            returnType,
            tuple.t2
        )
    }

    val foreignFun by -FOREIGN * FUN * IDENTIFIER * optionalWithStart(
        LPAREN,
        strictTrailing(strictlyTyped, COMMA) * RPAREN
    ) * zeroOrMore(strictParameterList) * optional(
        COLON * TypeGrammar.type
    ) * -FLOW * LiteralGrammar.string map { (token, id, first, rest, type, descriptor) ->
        val (colon, ret) = type ?: Tuple2(null, null)

        RLT.Function.Foreign(token.keyword(), id.identifier(), listOfNotNull(first?.let {
            RLT.TypedParameterTuple(it.t1.symbol(), it.t2.t1, it.t2.t2.symbol())
        }) + rest, colon?.symbol(), ret, descriptor)
    }

    override val rootParser by function
}
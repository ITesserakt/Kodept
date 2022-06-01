package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.combinators.unaryMinus
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import com.github.h0tk3y.betterParse.utils.Tuple3
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.CLASS
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.ENUM
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.STRUCT
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.TRAIT

object TopLevelGrammar : Grammar<RLT.TopLevelNode>() {
    private val functionStatement by FunctionGrammar

    val enumStatement by (ENUM * -(STRUCT or CLASS) * TYPE * LBRACE) and strictTrailing(
        TYPE, COMMA, atLeast = 1
    ) * RBRACE map { (enumToken, name, lb, entries, rb) ->
        RLT.Enum.Stack(
            RLT.Keyword(enumToken),
            RLT.UserSymbol.Type(name),
            lb.let(RLT::Symbol),
            NonEmptyList.fromListUnsafe(entries.map(RLT.UserSymbol::Type)),
            rb.let(RLT::Symbol)
        )
    }

    val traitStatement by TRAIT * TYPE * optionalWithStart(
        LBRACE, strictTrailing(ObjectLevelGrammar.traitLevel or FunctionGrammar) * RBRACE
    ) map { (traitToken, name, rest) ->
        val (lb, rest, rb) = rest?.let { Tuple3(it.t1, it.t2.t1, it.t2.t2) } ?: Tuple3(null, emptyList(), null)
        RLT.Trait(
            RLT.Keyword(traitToken),
            RLT.UserSymbol.Type(name),
            lb?.let(RLT::Symbol),
            rest,
            rb?.let(RLT::Symbol)
        )
    }

    val structStatement by STRUCT * TYPE * optionalWithStart(
        LPAREN, strictTrailing(IDENTIFIER * -COLON * TYPE, COMMA) * RPAREN
    ) * optionalWithStart(
        LBRACE, strictTrailing(FunctionGrammar) * RBRACE
    ) map { (structToken, name, allocated, rest) ->
        val (lp, alloc, rp) = allocated?.let { Tuple3(it.t1, it.t2.t1, it.t2.t2) } ?: Tuple3(null, emptyList(), null)
        val (lb, rest, rb) = rest?.let { Tuple3(it.t1, it.t2.t1, it.t2.t2) } ?: Tuple3(null, emptyList(), null)
        RLT.Struct(
            RLT.Keyword(structToken),
            RLT.UserSymbol.Type(name),
            lp?.let(RLT::Symbol),
            alloc.map { RLT.TypedParameter(RLT.UserSymbol.Identifier(it.t1), RLT.UserSymbol.Type(it.t2)) },
            rp?.let(RLT::Symbol),
            lb?.let(RLT::Symbol),
            rest,
            rb?.let(RLT::Symbol)
        )
    }

    override val rootParser: Parser<RLT.TopLevelNode> by structStatement or traitStatement or enumStatement or functionStatement
}
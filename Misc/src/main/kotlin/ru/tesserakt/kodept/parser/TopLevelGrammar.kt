package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.combinators.unaryMinus
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import com.github.h0tk3y.betterParse.utils.Tuple3
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.CLASS
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.ENUM
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.FOREIGN
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.STRUCT
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.TRAIT
import ru.tesserakt.kodept.lexer.ExpressionToken.Companion.TYPE_ALIAS

object TopLevelGrammar : Grammar<RLT.TopLevelNode>() {
    private val functionStatement by FunctionGrammar

    val enumStatement by (ENUM * -(STRUCT or CLASS) * TYPE * LBRACE) and strictTrailing(
        TYPE, COMMA, atLeast = 1
    ) * RBRACE map { (enumToken, name, lb, entries, rb) ->
        RLT.Enum.Stack(
            enumToken.keyword(),
            name.type(),
            lb.symbol(),
            NonEmptyList.fromListUnsafe(entries.map { it.type() }),
            rb.symbol()
        )
    }

    val traitStatement by TRAIT * TYPE * optionalWithStart(
        LBRACE, strictTrailing(ObjectLevelGrammar.traitLevel or FunctionGrammar) * RBRACE
    ) map { (traitToken, name, rest) ->
        val (lb, rest, rb) = rest?.let { Tuple3(it.t1, it.t2.t1, it.t2.t2) } ?: Tuple3(null, emptyList(), null)
        RLT.Trait(traitToken.keyword(), name.type(), lb?.symbol(), rest, rb?.symbol())
    }

    val structStatement by STRUCT * TYPE * optionalWithStart(
        LPAREN, strictTrailing(IDENTIFIER * -COLON * TypeGrammar, COMMA) * RPAREN
    ) * optionalWithStart(
        LBRACE, strictTrailing(FunctionGrammar) * RBRACE
    ) map { (structToken, name, allocated, rest) ->
        val (lp, alloc, rp) = allocated?.let { Tuple3(it.t1, it.t2.t1, it.t2.t2) } ?: Tuple3(null, emptyList(), null)
        val (lb, rest, rb) = rest?.let { Tuple3(it.t1, it.t2.t1, it.t2.t2) } ?: Tuple3(null, emptyList(), null)
        RLT.Struct(
            structToken.keyword(),
            name.type(),
            lp?.symbol(),
            alloc.map { RLT.TypedParameter(it.t1.identifier(), it.t2) },
            rp?.symbol(),
            lb?.symbol(),
            rest,
            rb?.symbol()
        )
    }

    val foreignType by FOREIGN * TYPE_ALIAS * TYPE * FLOW * LiteralGrammar.string map { (f, t, type, arrow, refersTo) ->
        RLT.ForeignType(f.keyword(), t.keyword(), type.type(), arrow.symbol(), refersTo)
    }

    override val rootParser: Parser<RLT.TopLevelNode> by structStatement or traitStatement or enumStatement or functionStatement or foreignType or FunctionGrammar.foreignFun
}
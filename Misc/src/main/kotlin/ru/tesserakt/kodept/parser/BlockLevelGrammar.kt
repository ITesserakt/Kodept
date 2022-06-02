package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.optional
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.keyword
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object BlockLevelGrammar : Grammar<RLT.BlockLevelNode>() {
    private val expression by OperatorGrammar
    private val functionStatement by FunctionGrammar

    val block = LBRACE and strictTrailing(this) * RBRACE map {
        RLT.Body.Block(RLT.Symbol(it.t1), it.t2, RLT.Symbol(it.t3))
    }
    val simple = FLOW * OperatorGrammar map {
        RLT.Body.Expression(RLT.Symbol(it.t1), it.t2)
    }
    val body = simple or block

    val varDecl by (VAL or VAR) * IDENTIFIER * optional(COLON) * optional(TYPE) map {
        val ctor = if (it.t1.type == VAL.token) RLT.Variable::Immutable
        else RLT.Variable::Mutable
        ctor(
            it.t1.keyword(),
            RLT.UserSymbol.Identifier(it.t2),
            it.t3?.let(RLT::Symbol),
            it.t4?.let(RLT.UserSymbol::Type)
        )
    }

    val initialization by varDecl * EQUALS * (block or OperatorGrammar) map { (decl, op, expr) ->
        RLT.InitializedAssignment(decl, RLT.Symbol(op), expr)
    }

    val assignment by TermGrammar *
            (PLUS_EQUALS or SUB_EQUALS or TIMES_EQUALS or DIV_EQUALS or MOD_EQUALS or POW_EQUALS or
                    OR_LOGIC_EQUALS or AND_LOGIC_EQUALS or
                    OR_BIT_EQUALS or AND_BIT_EQUALS or XOR_BIT_EQUALS) * (block or OperatorGrammar) map { (decl, op, expr) ->
        RLT.CompoundAssignment(decl, RLT.Symbol(op), expr)
    } or (TermGrammar * EQUALS * (block or OperatorGrammar) map { RLT.Assignment(it.t1, RLT.Symbol(it.t2), it.t3) })

    override val rootParser by functionStatement or initialization or assignment or expression or block
}
package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.optional
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.combinators.times
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.grammar.parser
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.identifier
import ru.tesserakt.kodept.core.keyword
import ru.tesserakt.kodept.core.symbol
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object BlockLevelGrammar : Grammar<RLT.BlockLevelNode>() {
    private val expression by OperatorGrammar
    private val functionStatement by parser { FunctionGrammar }

    val block by LBRACE and strictTrailing(this) * RBRACE map {
        RLT.Body.Block(it.t1.symbol(), it.t2, it.t3.symbol())
    }
    val simple by FLOW * OperatorGrammar map {
        RLT.Body.Expression(it.t1.symbol(), it.t2)
    }
    val body = simple or block

    val varDecl by (VAL or VAR) * IDENTIFIER * optional(COLON) * optional(TypeGrammar) map {
        val ctor = if (it.t1.type == VAL.token) RLT.Variable::Immutable
        else RLT.Variable::Mutable
        ctor(it.t1.keyword(), it.t2.identifier(), it.t3?.symbol(), it.t4)
    }

    val initialization by varDecl * EQUALS * (block or OperatorGrammar) map { (decl, op, expr) ->
        RLT.InitializedAssignment(decl, op.symbol(), expr)
    }

    val assignment by TermGrammar *
            (PLUS_EQUALS or SUB_EQUALS or TIMES_EQUALS or DIV_EQUALS or MOD_EQUALS or POW_EQUALS or
                    OR_LOGIC_EQUALS or AND_LOGIC_EQUALS or
                    OR_BIT_EQUALS or AND_BIT_EQUALS or XOR_BIT_EQUALS) * (block or OperatorGrammar) map { (decl, op, expr) ->
        RLT.CompoundAssignment(decl, op.symbol(), expr)
    } or (TermGrammar * EQUALS * (block or OperatorGrammar) map { RLT.Assignment(it.t1, it.t2.symbol(), it.t3) })

    override val rootParser by
    functionStatement or CodeFlowGrammar.whileStatement or initialization or assignment or expression or block
}
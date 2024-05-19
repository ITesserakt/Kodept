package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.keyword
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object CodeFlowGrammar : Grammar<RLT.ExpressionNode>() {
    val ifExpr by lazy {
        IF * OperatorGrammar * BlockLevelGrammar.body *
                zeroOrMore(ELIF * OperatorGrammar * BlockLevelGrammar.body) *
                optional(ELSE * BlockLevelGrammar.body) map { (ifToken, condition, block, elif, el) ->
            RLT.If(
                ifToken.keyword(),
                condition,
                block,
                elif.map { RLT.If.Elif(it.t1.keyword(), it.t2, it.t3) },
                el?.let { RLT.If.Else(it.t1.keyword(), it.t2) }
            )
        }
    }

    val whileStatement by lazy {
        WHILE * OperatorGrammar * BlockLevelGrammar.block map {
            RLT.While(it.t1.keyword(), it.t2, it.t3)
        }
    }

    override val rootParser by lazy { ifExpr or MatchGrammar }
}
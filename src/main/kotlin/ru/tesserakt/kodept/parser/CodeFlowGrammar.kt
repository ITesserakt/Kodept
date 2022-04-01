package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object CodeFlowGrammar : Grammar<AST.CodeFlowExpr>() {
    val block by BlockLevelGrammar.bracedDecls
    val simple by -FLOW * OperatorGrammar

    val ifExpr = -IF * OperatorGrammar * (simple or block) *
            zeroOrMore(-ELIF * OperatorGrammar * (simple or block)) *
            optional(-ELSE * (simple or block)) map { (condition, block, elif, el) ->
        AST.IfExpr(
            condition,
            block,
            elif.map { AST.IfExpr.ElifExpr(it.t1, it.t2) },
            el?.let { AST.IfExpr.ElseExpr(it) })
    }

    val whileExpr = -WHILE * OperatorGrammar * BlockLevelGrammar.bracedDecls map {
        AST.WhileExpr(it.t1, it.t2)
    }

    override val rootParser: Parser<AST.CodeFlowExpr> by ifExpr or whileExpr
}
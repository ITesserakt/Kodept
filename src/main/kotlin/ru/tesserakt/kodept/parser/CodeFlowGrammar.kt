package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object CodeFlowGrammar : Grammar<AST.CodeFlowExpr>() {
    val ifExpr by lazy {
        -IF * OperatorGrammar * BlockLevelGrammar.body *
                zeroOrMore(-ELIF * OperatorGrammar * BlockLevelGrammar.body) *
                optional(-ELSE * BlockLevelGrammar.body) map { (condition, block, elif, el) ->
            AST.IfExpr(
                condition,
                block,
                elif.map { AST.IfExpr.ElifExpr(it.t1, it.t2) },
                el?.let { AST.IfExpr.ElseExpr(it) })
        }
    }

    val whileExpr by lazy {
        -WHILE * OperatorGrammar * BlockLevelGrammar.bracedDecls map {
            AST.WhileExpr(it.t1, it.t2)
        }
    }

    override val rootParser: Parser<AST.CodeFlowExpr> by lazy { ifExpr or whileExpr }
}
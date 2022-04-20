package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.toCodePoint

object CodeFlowGrammar : Grammar<AST.CodeFlowExpr>() {
    val ifExpr by lazy {
        IF * OperatorGrammar * BlockLevelGrammar.body *
                zeroOrMore(ELIF * OperatorGrammar * BlockLevelGrammar.body) *
                optional(ELSE * BlockLevelGrammar.body) map { (ifToken, condition, block, elif, el) ->
            AST.IfExpr(
                condition,
                block,
                elif.map { AST.IfExpr.ElifExpr(it.t2, it.t3, it.t1.toCodePoint()) },
                el?.let { AST.IfExpr.ElseExpr(it.t2, it.t1.toCodePoint()) },
                ifToken.toCodePoint()
            )
        }
    }

    val whileExpr by lazy {
        WHILE * OperatorGrammar * BlockLevelGrammar.bracedDecls map {
            AST.WhileExpr(it.t2, it.t3, it.t1.toCodePoint())
        }
    }

    override val rootParser: Parser<AST.CodeFlowExpr> by lazy { ifExpr or whileExpr }
}
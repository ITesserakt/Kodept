package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object TermGrammar : Grammar<AST.Term>() {
    val variableReference by IDENTIFIER use { AST.UnresolvedReference(text) }

    val functionCall by variableReference * -LPAREN * trailing(
        OperatorGrammar,
        COMMA
    ) * -RPAREN use { AST.UnresolvedFunctionCall(t1, t2) }

    val callChain by zeroOrMore((functionCall or variableReference) * -DOT) * (functionCall or variableReference) use {
        AST.TermChain(t1 + listOf(t2))
    }

    override val rootParser by callChain map { if (it.terms.size == 1) it.terms.first() else it }
}
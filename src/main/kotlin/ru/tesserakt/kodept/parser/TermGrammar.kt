package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.lexer.ExpressionToken.*

object TermGrammar : Grammar<AST.Term>() {
    val variableReference by IDENTIFIER use { AST.UnresolvedReference(text) }

    val functionCall by variableReference * -LPAREN * separatedTerms(
        OperatorGrammar,
        COMMA,
        true
    ) * -RPAREN use { AST.UnresolvedFunctionCall(t1, t2) }

    val callChain by separatedTerms(functionCall or variableReference, DOT) use { AST.TermChain(this) }

    override val rootParser by callChain map { if (it.terms.size == 1) it.terms.first() else it }
}
package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.AST
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.toCodePoint
import ru.tesserakt.kodept.trailing

object TermGrammar : Grammar<AST.Term>() {
    val variableReference by IDENTIFIER use { AST.UnresolvedReference(text, toCodePoint()) }

    val functionCall by variableReference * -LPAREN * trailing(
        OperatorGrammar,
        COMMA
    ) * -RPAREN use { AST.UnresolvedFunctionCall(t1, t2) }

    val callChain by zeroOrMore((functionCall or variableReference) * -DOT) * (functionCall or variableReference) use {
        AST.TermChain(NonEmptyList.fromListUnsafe(t1 + listOf(t2)))
    }

    override val rootParser by callChain map { if (it.terms.size == 1) it.terms.first() else it }
}
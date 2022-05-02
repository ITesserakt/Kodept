package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.grammar.Grammar
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.lexer.ExpressionToken.*
import ru.tesserakt.kodept.lexer.toCodePoint

object TermGrammar : Grammar<AST.Term>() {
    val variableReference by IDENTIFIER use { AST.Reference(text, toCodePoint()) }

    val typeReference by TYPE use { AST.TypeReference(AST.TypeExpression(text, toCodePoint())) }

    val scopeResolution by optional(DOUBLE_COLON) * oneOrMore(typeReference * -DOUBLE_COLON) use {
        AST.ResolutionContext(t1 != null, NonEmptyList.fromListUnsafe(t2))
    }

    val functionCall by variableReference * -LPAREN * trailing(
        OperatorGrammar,
        COMMA
    ) * -RPAREN use { AST.FunctionCall(t1, t2) }

    val callChain by zeroOrMore((functionCall or variableReference) * -DOT) * (functionCall or variableReference) use {
        AST.TermChain(NonEmptyList.fromListUnsafe(t1 + listOf(t2)))
    }

    override val rootParser by (scopeResolution * functionCall use { t2.copy(resolutionContext = t1) }) or
            (callChain map { if (it.terms.size == 1) it.terms.first() else it }) or
            (optional(scopeResolution) * typeReference use { t2.copy(resolutionContext = t1) })
}
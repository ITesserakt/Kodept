package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.*
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.lexer.ExpressionToken

inline fun <reified T> trailing(
    other: Parser<T>,
    separator: Parser<*> = ExpressionToken.SEMICOLON or ExpressionToken.NEWLINE,
    atLeast: Int = 0,
) = when (atLeast) {
    0 -> zeroOrMore(other * -separator) * optional(other * -optional(separator)) use {
        t1 + listOfNotNull(t2)
    }
    1 -> (zeroOrMore(other * -separator) * other use { t1 + listOf(t2) }) or (oneOrMore(other * -separator))
    else -> (((atLeast - 1) timesOrMore other * -separator) * other use { t1 + listOf(t2) }) or
            (atLeast timesOrMore (other * -separator))
}
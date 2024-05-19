package ru.tesserakt.kodept.core

import arrow.eval.Eval
import com.github.h0tk3y.betterParse.lexer.CharToken
import com.github.h0tk3y.betterParse.lexer.LiteralToken
import com.github.h0tk3y.betterParse.lexer.TokenMatch

fun TokenMatch.keyword() = RLT.Keyword(
    (type as? LiteralToken)?.text?.let(Eval.Companion::now) ?: Eval.later { text },
    toCodePoint()
)

fun TokenMatch.symbol() = RLT.Symbol(
    when (val t = type) {
        is LiteralToken -> t.text
        is CharToken -> t.text.toString()
        else -> throw IllegalArgumentException("Type should be either literal or char token")
    }.let(Eval.Companion::now),
    toCodePoint(),
    requireNotNull(type.name) { "Symbol name should be not null" }
)

fun TokenMatch.type() = RLT.UserSymbol.Type(
    Eval.later { text },
    toCodePoint()
)

fun TokenMatch.identifier() = RLT.UserSymbol.Identifier(
    Eval.later { text },
    toCodePoint()
)
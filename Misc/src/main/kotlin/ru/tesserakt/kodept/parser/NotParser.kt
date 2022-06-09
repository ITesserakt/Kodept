package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.lexer.TokenMatchesSequence
import com.github.h0tk3y.betterParse.parser.ErrorResult
import com.github.h0tk3y.betterParse.parser.Parsed
import com.github.h0tk3y.betterParse.parser.Parser

private object Inversion : ErrorResult()

class NotParser(private val inner: Parser<*>) : Parser<Unit> {
    override fun tryParse(tokens: TokenMatchesSequence, fromPosition: Int) =
        when (inner.tryParse(tokens, fromPosition)) {
            is Parsed -> Inversion
            is ErrorResult -> parsed(Unit, fromPosition)
        }
}

operator fun Parser<*>.not(): Parser<Unit> = NotParser(this)
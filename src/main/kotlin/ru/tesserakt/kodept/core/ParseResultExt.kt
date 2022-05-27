package ru.tesserakt.kodept.core

import arrow.core.left
import arrow.core.right
import com.github.h0tk3y.betterParse.parser.ErrorResult
import com.github.h0tk3y.betterParse.parser.ParseResult
import com.github.h0tk3y.betterParse.parser.Parsed

@PublishedApi
internal data class ParsedValue<T>(override val value: T, override val nextPosition: Int) : Parsed<T>()

fun <T> parsed(value: T, nextPosition: Int): Parsed<T> = ParsedValue(value, nextPosition)

inline fun <T, U> ParseResult<T>.map(f: (T) -> U) = when (this) {
    is Parsed -> ParsedValue(f(value), nextPosition)
    is ErrorResult -> this
}

fun <T> ParseResult<T>.toEither() = when (this) {
    is Parsed -> value.right()
    is ErrorResult -> left()
}

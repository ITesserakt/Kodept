package ru.tesserakt.kodept.core

import arrow.core.left
import arrow.core.right
import com.github.h0tk3y.betterParse.parser.ErrorResult
import com.github.h0tk3y.betterParse.parser.ParseResult
import com.github.h0tk3y.betterParse.parser.Parsed

fun <T, U> ParseResult<T>.map(f: (T) -> U) = when (this) {
    is Parsed -> object : Parsed<U>() {
        override val nextPosition: Int = this@map.nextPosition
        override val value: U = f(this@map.value)
    }
    is ErrorResult -> this
}

fun <T> ParseResult<T>.toEither() = when (this) {
    is Parsed -> value.right()
    is ErrorResult -> left()
}

fun <T, U> ParseResult<T>.catch(f: (ErrorResult) -> U) = toEither().mapLeft(f)
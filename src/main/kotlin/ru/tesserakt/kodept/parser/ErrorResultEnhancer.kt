package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.parser.*

class ErrorResultEnhancer(private val file: String, private val unknownErrorStrategy: Strategy) {
    abstract class Strategy {
        abstract fun handleUnknown(errorResult: ErrorResult): Nothing
    }

    fun ErrorResult.enhance(): Nothing = when (this) {
        is UnparsedRemainder -> TODO()
        is MismatchedToken -> TODO()
        is NoMatchingToken -> TODO()
        is UnexpectedEof -> TODO()
        is AlternativesFailure -> TODO()
        else -> unknownErrorStrategy.handleUnknown(this)
    }
}
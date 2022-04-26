package ru.tesserakt.kodept.error

import arrow.core.NonEmptyList
import com.github.h0tk3y.betterParse.lexer.Token
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import com.github.h0tk3y.betterParse.parser.ErrorResult

sealed class SyntaxError(final override val code: String, override val message: String) : ReportMessage {
    init {
        require(code.startsWith("KSyE"))
    }

    data class MismatchedToken(val expected: NonEmptyList<Token>, val actual: TokenMatch) : SyntaxError("KSyE1",
        "Expected ${expected.joinToString { it.name ?: "<UNDEFINED>" }}, found ${actual.text}")

    object UnparsedRemainder : SyntaxError("KSyE2", "Could not parse further. Check your syntax")

    data class UnknownToken(val token: TokenMatch) : SyntaxError("KSyE3", "Unknown token: ${token.text}")

    data class UnexpectedEOF(val expected: NonEmptyList<Token>) : SyntaxError("KSyE4", "")

    data class Common(val error: ErrorResult) : SyntaxError("KSyE5", error.toString())
}

package ru.tesserakt.kodept.error

import arrow.core.NonEmptyList
import com.github.h0tk3y.betterParse.lexer.*
import com.github.h0tk3y.betterParse.parser.ErrorResult

private fun Token.pretty() = when (this) {
    is CharToken -> "`${text}`"
    is LiteralToken -> "`${text}`"
    is RegexToken -> name?.let { "<${it}>" }
    else -> name
}

sealed class SyntaxError(final override val code: String, override val message: String) : ReportMessage {
    init {
        require(code.startsWith("KSyE"))
    }

    data class MismatchedToken(val expected: NonEmptyList<Token>, val actual: TokenMatch) : SyntaxError(
        "KSyE1",
        "Expected ${expected.joinToString { it.pretty() ?: "<UNDEFINED>" }}, found ${actual.type.pretty() ?: actual.text}"
    )

    object UnparsedRemainder : SyntaxError("KSyE2", "Could not parse further. Check your syntax")

    data class UnknownToken(val token: TokenMatch) : SyntaxError("KSyE3", "Unknown token: ${token.text}")

    data class UnexpectedEOF(val expected: NonEmptyList<Token>) :
        SyntaxError("KSyE4", "Expected ${expected.joinToString { it.pretty() ?: "<UNDEFINED>" }}, found EOF")

    data class Common(val error: ErrorResult) : SyntaxError("KSyE5", error.toString())
}

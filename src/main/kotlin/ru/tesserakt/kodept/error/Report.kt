package ru.tesserakt.kodept.error

import arrow.core.NonEmptyList
import ru.tesserakt.kodept.lexer.CodePoint

data class Report(
    val file: String,
    val point: NonEmptyList<CodePoint>,
    val severity: Severity,
    val message: ReportMessage,
) {
    enum class Severity { NOTE, WARNING, ERROR }

    override fun toString() = "$file:$point:\n$severity[${message.code}]: ${message.message}"
}

interface ReportMessage {
    val code: String
    val message: String
    val additionalMessage: String get() = "here"
}
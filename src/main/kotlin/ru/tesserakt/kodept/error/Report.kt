package ru.tesserakt.kodept.error

import ru.tesserakt.kodept.lexer.CodePoint

data class Report(val file: String, val line: CodePoint, val severity: Severity, val message: ReportMessage) {
    enum class Severity { NOTE, WARNING, ERROR }
}

sealed interface ReportMessage {
    val code: String
    val message: String
}
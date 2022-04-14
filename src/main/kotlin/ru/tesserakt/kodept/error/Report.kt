package ru.tesserakt.kodept.error

import ru.tesserakt.kodept.lexer.CodePoint

data class Report(val file: String, val line: CodePoint, val severity: Severity, val message: SemanticError) {
    enum class Severity { NOTE, WARNING, ERROR }
}
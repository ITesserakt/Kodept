package ru.tesserakt.kodept.error

import arrow.core.NonEmptyList
import ru.tesserakt.kodept.core.CodePoint

data class Report(
    val file: String?,
    val point: NonEmptyList<CodePoint>?,
    val severity: Severity,
    val message: ReportMessage,
) {
    enum class Severity { NOTE, WARNING, ERROR, CRASH }

    override fun toString(): String {
        val fileStr = file?.let { "\n$it:" } ?: ""
        val pointStr = (point?.head ?: CodePoint(0, 0)).takeIf { file != null } ?: ""
        return "$severity[${message.code}]: ${message.message}$fileStr$pointStr"
    }
}

interface ReportMessage {
    val code: String
    val message: String
    val additionalMessage: String get() = "here"
}
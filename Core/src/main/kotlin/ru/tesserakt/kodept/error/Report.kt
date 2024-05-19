package ru.tesserakt.kodept.error

import arrow.core.NonEmptyList
import ru.tesserakt.kodept.core.CodePoint
import ru.tesserakt.kodept.core.Filepath
import java.io.File

data class Report(
    val file: Filepath?,
    val point: NonEmptyList<CodePoint>?,
    val severity: Severity,
    val message: ReportMessage,
) {
    enum class Severity { NOTE, WARNING, ERROR, CRASH }

    override fun toString(): String {
        val fileStr = file?.asFile?.relativeToOrSelf(File("./").absoluteFile)?.let { "\n$it" } ?: ""
        val pointStr = point?.head?.let { ":$it" }.takeIf { file != null } ?: ""
        return "$severity[${message.code}]: ${message.message}$fileStr$pointStr"
    }

    companion object {
        context (Filepath)
                operator fun invoke(point: NonEmptyList<CodePoint>?, severity: Severity, message: ReportMessage) =
            Report(this@Filepath, point, severity, message)
    }
}

interface ReportMessage {
    val code: String
    val message: String
    val additionalMessage: String get() = "here"
}
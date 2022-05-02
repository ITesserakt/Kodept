package ru.tesserakt.kodept.error

import ru.tesserakt.kodept.core.FileRelative
import ru.tesserakt.kodept.core.StringContent
import ru.tesserakt.kodept.core.mapWithFilename
import kotlin.math.log10

class ReportProcessor private constructor(private val surrounding: Int, private val pointer: String) {
    class Builder {
        var surrounding: Int = 3
        var defaultErrorPointer = "^---"

        fun build() = ReportProcessor(surrounding, defaultErrorPointer)
    }

    companion object {
        operator fun invoke(block: Builder.() -> Unit = {}) = Builder().apply(block).build()
    }

    private var linesMap: Sequence<FileRelative<Sequence<String>>>? = null

    context (StringContent)
    fun cacheText() {
        linesMap = text.mapWithFilename { it.lineSequence() }
    }

    context (StringContent)
    fun processReport(report: Report): String {
        val maxIndexLength = report.point?.maxOf { log10(it.line.toFloat()).toInt() + 1 } ?: 0
        val linesMap = linesMap ?: text.mapWithFilename { it.lineSequence() }
        val codeWindows = report.point.orEmpty().map { point ->
            val stream = linesMap.first { it.filename == report.file }.value
            val from = (point.line - surrounding).coerceAtLeast(0)

            stream.drop(from).take(surrounding).withIndex().joinToString("\n") { (index, str) ->
                val realIndex = index + from + 1
                val lineNumber = "    %${maxIndexLength}d | ".format(realIndex)
                lineNumber + if (realIndex == point.line)
                    "$str\n${" ".repeat(point.position + lineNumber.length - 1)}^--- ${report.message.additionalMessage}"
                else str
            }
        }

        return "$report\n${codeWindows.joinToString("\n")}"
    }
}
package ru.tesserakt.kodept.error

import ru.tesserakt.kodept.core.ProgramCodeHolder
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

    context (ProgramCodeHolder)
    fun processReport(report: Report): String {
        val maxIndexLength = report.point?.maxOf { log10(it.line.toFloat()).toInt() + 1 } ?: 0
        val codeWindows = report.point.orEmpty().map { point ->
            val from = (point.line - surrounding).coerceAtLeast(0)
            val stream = get(report.file).linesRange(from..surrounding)

            stream.withIndex().joinToString("\n") { (index, str) ->
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
package ru.tesserakt.kodept.error

import ru.tesserakt.kodept.core.ProgramCodeHolder
import kotlin.math.log10

class ReportProcessor private constructor(
    private val surrounding: Int,
    private val pointer: String,
    private val longPointer: String,
) {
    class Builder {
        var surrounding: Int = 3
        var defaultErrorPointer = "^---"
        var defaultLongErrorPointer = "^"

        fun build() = ReportProcessor(surrounding, defaultErrorPointer, defaultLongErrorPointer)
    }

    companion object {
        operator fun invoke(block: Builder.() -> Unit = {}) = Builder().apply(block).build()
    }

    context (ProgramCodeHolder)
    fun processReport(report: Report): String {
        val maxIndexLength = report.point?.maxOf { log10(it.line.toFloat()).toInt() + 1 } ?: 0
        val codeWindows = report.point.orEmpty().map { point ->
            val from = (point.line - surrounding).coerceAtLeast(0)
            val stream = get(report.file).linesRange(from - 1..from + surrounding)

            stream.withIndex().joinToString("\n") { (index, str) ->
                val realIndex = index + from
                val lineNumber = "    %${maxIndexLength}d | ".format(realIndex)
                lineNumber + if (realIndex == point.line) {
                    val ident = " ".repeat(point.position + lineNumber.length - 1)
                    val repetitions = point.length / longPointer.length
                    val useLongPointer = repetitions != 1
                    val pointee =
                        if (useLongPointer) longPointer.repeat(repetitions) + " -"
                        else pointer
                    "$str\n${ident}${pointee} ${report.message.additionalMessage}"
                } else str
            }
        }

        val str = "$report\n${codeWindows.joinToString("\n")}"
        return if (report.message is CompilerCrash)
            str + "\n" + report.message.stackTraceToString()
        else str
    }
}
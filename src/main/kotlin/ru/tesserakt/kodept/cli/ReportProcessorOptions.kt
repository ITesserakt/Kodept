package ru.tesserakt.kodept.cli

import arrow.core.IorNel
import arrow.core.NonEmptyList
import com.github.ajalt.clikt.parameters.groups.OptionGroup
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.choice
import com.github.ajalt.clikt.parameters.types.int
import com.github.ajalt.clikt.parameters.types.restrictTo
import io.github.oshai.kotlinlogging.KLogger
import ru.tesserakt.kodept.core.ProgramCodeHolder
import ru.tesserakt.kodept.error.CompilerCrash
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportProcessor

class ReportProcessorOptions : OptionGroup(name = "Options for reports") {
    private val surrounding by option(help = "Show N surrounding lines of code in report", metavar = "N").int()
        .restrictTo(0)
        .default(0)
    private val pointer by option(help = "Style of pointer in report").choice("^--", "^~~").default("^--")
    private val longPointer by option(help = "Style of long pointer in report").choice("^", "T").default("^")

    val processor
        get() = ReportProcessor {
            surrounding = this@ReportProcessorOptions.surrounding
            defaultErrorPointer = pointer
            defaultLongErrorPointer = longPointer
        }
}

context (ReportProcessor, ProgramCodeHolder, KLogger)
        private fun logReports(reports: NonEmptyList<Report>) = reports.forEach {
    when (it.severity) {
        Report.Severity.NOTE -> info { processReport(it) }
        Report.Severity.WARNING -> warn { processReport(it) }
        Report.Severity.ERROR -> error { processReport(it) }
        Report.Severity.CRASH -> if (it.message is CompilerCrash)
            error(it.message as CompilerCrash) { processReport(it) }
        else error { processReport(it) }
    }
}

context (ReportProcessor, ProgramCodeHolder, KLogger)
fun <T> IorNel<Report, T>.printReportsOr(f: (T) -> String) =
    this.fold({ logReports(it) }, {
        info { "Completed without any report: ${f(it)}" }
    }, { r, it ->
        warn { "Completed with following reports: ${f(it)}" }
        logReports(r)
    })
package ru.tesserakt.kodept.error

import arrow.core.NonEmptyList

class ReportCollector {
    private val reports = mutableListOf<Report>()
    private var errors = false

    private fun Report.isError() = severity == Report.Severity.ERROR || severity == Report.Severity.CRASH

    fun Report.report() {
        errors = errors || isError()
        reports += this
    }

    fun Iterable<Report>.report() {
        errors = errors || any { it.isError() }
        reports += this
    }

    inline fun <T> Iterable<T>.reportEach(f: (T) -> Report) = map(f).report()

    fun Sequence<Report>.report() {
        errors = errors || any { it.isError() }
        reports += this
    }

    fun <T> Sequence<T>.reportEach(f: (T) -> Report) = map(f).report()

    val collectedReports get() = reports
    val definitelyCollected get() = NonEmptyList.fromListUnsafe(collectedReports)
    val hasReports get() = reports.isNotEmpty()
    val hasErrors get() = errors
}
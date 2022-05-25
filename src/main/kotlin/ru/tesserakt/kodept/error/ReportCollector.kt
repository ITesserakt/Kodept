package ru.tesserakt.kodept.error

import arrow.core.NonEmptyList

class ReportCollector {
    private val reports = mutableListOf<Report>()

    fun Report.report() {
        reports += this
    }

    fun Iterable<Report>.report() {
        reports += this
    }

    fun <T> Iterable<T>.reportEach(f: (T) -> Report) {
        map(f).report()
    }

    val collectedReports get() = reports.toList()

    val definitelyCollected get() = NonEmptyList.fromListUnsafe(collectedReports)
}
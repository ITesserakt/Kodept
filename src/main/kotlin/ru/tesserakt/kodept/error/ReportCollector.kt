package ru.tesserakt.kodept.error

open class ReportCollector {
    private val reports = mutableListOf<Report>()

    protected fun Report.report() {
        reports += this
    }

    protected fun Iterable<Report>.report() {
        reports += this
    }

    protected fun <T> Iterable<T>.reportEach(f: (T) -> Report) {
        map(f).report()
    }

    val collectedReports get() = reports.toList()
}
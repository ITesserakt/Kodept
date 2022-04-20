package ru.tesserakt.kodept.analyzer

import arrow.core.Eval
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector

abstract class Analyzer : ReportCollector() {
    open fun analyze(files: Sequence<AST>): Eval<List<Report>> = Eval.later {
        files.map { analyzeIndependently(it) }.toList()
        collectedReports
    }

    open fun analyzeIndependently(ast: AST) {}
}
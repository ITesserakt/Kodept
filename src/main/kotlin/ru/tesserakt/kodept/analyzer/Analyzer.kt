package ru.tesserakt.kodept.analyzer

import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.error.ReportCollector

abstract class Analyzer : ReportCollector() {
    abstract fun analyzeIndependently(ast: AST)
}
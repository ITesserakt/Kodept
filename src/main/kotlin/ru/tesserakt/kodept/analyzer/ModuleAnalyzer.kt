package ru.tesserakt.kodept.analyzer

import arrow.core.NonEmptyList
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.visitor.ModuleCollector

class ModuleAnalyzer : Analyzer() {
    private val collector = ModuleCollector()

    override fun analyzeIndependently(ast: AST) {
        val moduleDecls = collector.collect(ast.root)
        collector.collectedReports.report()

        moduleDecls.groupBy { it.name }.values
            .filter { it.size > 1 }
            .map { NonEmptyList.fromListUnsafe(it) }
            .reportEach { modules ->
                Report(ast.fileName,
                    modules.head.coordinates,
                    Report.Severity.ERROR,
                    SemanticError.DuplicatedModules(modules.map { it.coordinates to it.name }))
            }
    }
}
package ru.tesserakt.kodept.analyzer

import arrow.core.NonEmptyList
import arrow.core.nel
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.error.SemanticWarning
import ru.tesserakt.kodept.visitor.ModuleCollector

class ModuleAnalyzer : Analyzer() {
    private val collector = ModuleCollector()

    override fun analyzeIndependently(ast: AST) {
        val moduleDecls = collector.collect(ast.root)
        collector.collectedReports.report()

        val module = moduleDecls.firstOrNull()
        if (moduleDecls.size == 1 && module != null && !module.global)
            Report(ast.fileName,
                module.coordinates.shiftHorizontally(6 + module.name.length + 2).nel(),
                Report.Severity.WARNING,
                SemanticWarning.NonGlobalSingleModule(module.name)).report()

        moduleDecls.groupBy { it.name }.values
            .filter { it.size > 1 }
            .map { NonEmptyList.fromListUnsafe(it) }
            .reportEach { modules ->
                Report(ast.fileName,
                    modules.map { it.coordinates },
                    Report.Severity.ERROR,
                    SemanticError.DuplicatedModules(modules.map { it.coordinates to it.name }))
            }
    }
}
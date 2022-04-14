package ru.tesserakt.kodept.analyzer

import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.parser.AST
import ru.tesserakt.kodept.visitor.ModuleCollector

class ModuleAnalyzer {
    private val collector = ModuleCollector()

    fun analyze(files: Sequence<AST>) =
        files.map { collector.collect(it.root) to it.fileName }.flatMap { (list, file) ->
            (list - list.distinctBy { it.name }.toSet()).map {
                Report(file,
                    it.coordinates,
                    Report.Severity.ERROR,
                    SemanticError.DuplicatedModules(it.name))
            }
        }
}
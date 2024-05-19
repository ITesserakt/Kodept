package ru.tesserakt.kodept.traversal

import arrow.core.nonEmptyListOf
import arrow.core.raise.EagerEffect
import arrow.core.raise.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticWarning

object ModuleUniquenessAnalyzer : Analyzer() {
    override fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit> = eagerEffect {
        val modules = ast.fastFlatten().filterIsInstance<AST.ModuleDecl>().filter { !it.global }.toList()
        if (modules.size != 1) return@eagerEffect
        val head = modules.first()
        val rlt = head.rlt as RLT.Module.Ordinary

        Report(
            ast.filepath,
            nonEmptyListOf(rlt.lbrace, rlt.rbrace).map { it.position },
            Report.Severity.WARNING,
            SemanticWarning.NonGlobalSingleModule(head.name)
        ).report()
    }
}
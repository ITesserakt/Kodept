package ru.tesserakt.kodept.traversal

import arrow.core.NonEmptyList
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.nonEmptyListOf
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.accessRLT
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.error.SemanticWarning

val moduleNameAnalyzer = object : Analyzer() {
    override fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit> = eagerEffect {
        ast.fastFlatten().filterIsInstance<AST.ModuleDecl>()
            .groupBy { it.name }
            .values
            .filter { it.size > 1 }
            .map { NonEmptyList.fromListUnsafe(it) }
            .reportEach { modules ->
                Report(
                    ast.filepath,
                    modules.map { it.accessRLT<RLT.Module>()!!.id.position },
                    Report.Severity.ERROR,
                    SemanticError.DuplicatedModules(modules.head.name)
                )
            }
    }
}

val moduleUniquenessAnalyzer = object : Analyzer() {
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
package ru.tesserakt.kodept.traversal

import arrow.core.NonEmptyList
import arrow.core.raise.EagerEffect
import arrow.core.raise.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.accessRLT
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError

object ModuleNameAnalyzer : Analyzer() {
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


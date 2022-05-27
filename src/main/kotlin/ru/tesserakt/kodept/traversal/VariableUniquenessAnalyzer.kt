package ru.tesserakt.kodept.traversal

import arrow.core.NonEmptyList
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.rlt
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError

val variableUniqueness = object : Analyzer() {
    override fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit> = eagerEffect {
        val blocks = ast.fastFlatten { it is AST.ExpressionList }

        blocks.flatMap {
            it.children().filterIsInstance<AST.VariableDecl>().groupBy(AST.VariableDecl::name).values
        }.filter { it.size != 1 }.reportEach { vars ->
            Report(
                ast.filename,
                NonEmptyList.fromListUnsafe(vars.map { it.rlt.id.position }),
                Report.Severity.ERROR,
                SemanticError.DuplicatedVariable(vars.first().name.name)
            )
        }
    }
}
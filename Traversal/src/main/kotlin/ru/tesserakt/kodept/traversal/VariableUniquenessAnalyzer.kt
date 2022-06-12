package ru.tesserakt.kodept.traversal

import arrow.core.NonEmptyList
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.accessRLT
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError

val variableUniqueness = object : Analyzer() {
    override fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit> = eagerEffect {
        val blocks = ast.fastFlatten { it is AST.ExpressionList }

        blocks.flatMap { node ->
            node.children().filterIsInstance<AST.InitializedVar>().groupBy { it.reference }.values
        }.filter { it.size != 1 }.reportEach { vars ->
            val points = vars.mapNotNull { it.accessRLT<RLT.Variable>()?.id?.position }

            Report(
                ast.filepath,
                NonEmptyList.fromList(points).orNull(),
                Report.Severity.ERROR,
                SemanticError.DuplicatedVariable(vars.first().reference.name)
            )
        }
    }
}
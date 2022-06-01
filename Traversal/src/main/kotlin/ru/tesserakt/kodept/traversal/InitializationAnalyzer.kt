package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.nonEmptyListOf
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError

object InitializationAnalyzer : Analyzer() {
    init {
        dependsOn(InitializationTransformer)
    }

    override fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit> = eagerEffect {
        val refs = ast.fastFlatten().filterIsInstance<AST.ResolvedReference>()

        refs.forEach {
            when (it.referral) {
                is AST.FunctionDecl -> Unit
                is AST.InferredParameter -> Unit
                is AST.Parameter -> Unit
                is AST.InitializedVar -> Unit
                is AST.VariableDecl -> shift(
                    UnrecoverableError(
                        Report(
                            ast.filename,
                            nonEmptyListOf(it.referral.rlt.position, it.rlt.position),
                            Report.Severity.ERROR,
                            SemanticError.UninitializedUsage(it.referral.name)
                        )
                    )
                )
            }
        }
    }
}
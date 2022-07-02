package ru.tesserakt.kodept.traversal

import arrow.core.NonEmptyList
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.nel
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.accessRLT
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticNote
import ru.tesserakt.kodept.error.SemanticWarning

private inline fun <reified N : AST.Node> Sequence<AST.Node>.generateReports(
    reportSink: ReportCollector,
    noinline predicate: (N) -> Boolean,
    noinline action: (N) -> Report,
) = with(reportSink) {
    filterIsInstance<N>()
        .filter(predicate)
        .reportEach(action)
}

object EmptyBlockAnalyzer : Analyzer() {
    override fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit> = eagerEffect {
        val emptyStructures = ast.fastFlatten {
            it is AST.StructDecl && it.alloc.isEmpty() ||
                    it is AST.StructDecl && it.rest.isEmpty() ||
                    it is AST.EnumDecl && it.enumEntries.isEmpty() ||
                    it is AST.AbstractFunctionDecl && it.params.isEmpty() ||
                    it is AST.FunctionDecl && it.params.isEmpty() ||
                    it is AST.ExpressionList && AST.TupleLiteral.unit in it.expressions ||
                    it is AST.TraitDecl && it.rest.isEmpty()
        }

        emptyStructures.generateReports<AST.StructDecl>(
            this@analyze,
            { it.accessRLT<RLT.Struct>()?.lparen != null && it.alloc.isEmpty() },
            {
                Report(
                    ast.filepath,
                    it.accessRLT<RLT.Struct>()?.lparen?.position?.nel(),
                    Report.Severity.WARNING,
                    SemanticWarning.EmptyParameterList(it.name)
                )
            })

        emptyStructures.generateReports<AST.TraitDecl>(this@analyze, { it.accessRLT<RLT.Trait>()?.lbrace != null }) {
            Report(
                ast.filepath,
                it.accessRLT<RLT.Trait>()?.lbrace?.position?.nel(),
                Report.Severity.WARNING,
                SemanticWarning.EmptyBlock(it.name)
            )
        }

        emptyStructures.generateReports<AST.StructDecl>(
            this@analyze,
            { it.accessRLT<RLT.Struct>()?.lbrace != null && it.rest.isEmpty() }) {
            Report(
                ast.filepath,
                it.accessRLT<RLT.Struct>()?.lbrace?.position?.nel(),
                Report.Severity.WARNING,
                SemanticWarning.EmptyBlock(it.name)
            )
        }

        emptyStructures.generateReports<AST.EnumDecl>(this@analyze, { it.accessRLT<RLT.Enum>()?.lbrace != null }) {
            Report(
                ast.filepath,
                it.accessRLT<RLT.Enum>()?.lbrace?.position?.nel(),
                Report.Severity.WARNING,
                SemanticWarning.ZeroEnumEntries(it.name)
            )
        }

        emptyStructures.generateReports<AST.AbstractFunctionDecl>(
            this@analyze,
            { decl -> decl.accessRLT<RLT.Function.Abstract>()?.params.orEmpty().any { it.params.isEmpty() } }) {
            Report(
                ast.filepath,
                NonEmptyList.fromListUnsafe(
                    it.accessRLT<RLT.Function.Abstract>()?.params.orEmpty()
                        .filter { it.params.isEmpty() }
                        .map { it.lparen.position }),
                Report.Severity.WARNING,
                SemanticWarning.EmptyParameterList(it.name)
            )
        }

        emptyStructures.generateReports<AST.FunctionDecl>(
            this@analyze,
            { decl -> decl.accessRLT<RLT.Function.Bodied>()?.params.orEmpty().any { it.params.isEmpty() } }) {
            Report(
                ast.filepath,
                NonEmptyList.fromListUnsafe(
                    it.accessRLT<RLT.Function.Bodied>()?.params.orEmpty().filter { it.params.isEmpty() })
                    .map { it.lparen.position },
                Report.Severity.WARNING,
                SemanticWarning.EmptyParameterList(it.name)
            )
        }

        emptyStructures.generateReports<AST.ExpressionList>(this@analyze, { true }) {
            Report(
                ast.filepath,
                it.rlt.position.nel(),
                Report.Severity.NOTE,
                SemanticNote.EmptyComputationBLock
            )
        }
    }
}
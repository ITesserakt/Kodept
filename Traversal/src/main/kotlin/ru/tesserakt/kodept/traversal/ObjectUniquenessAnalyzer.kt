package ru.tesserakt.kodept.traversal

import arrow.core.NonEmptyList
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError

val objectUniqueness = object : Analyzer() {
    override fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit> = eagerEffect {
        val blocks = ast.fastFlatten {
            it is AST.ModuleDecl ||
                    it is AST.StructDecl ||
                    it is AST.TraitDecl ||
                    it is AST.EnumDecl ||
                    it is AST.ExpressionList ||
                    it is AST.AbstractFunctionDecl ||
                    it is AST.ForeignFunctionDecl ||
                    it is AST.FunctionDecl
        }

        val duplicates = blocks.flatMap {
            when (it) {
                is AST.ModuleDecl -> it.children().groupBy(AST.Named::name).values
                is AST.StructDecl -> it.rest.groupBy(AST.Named::name).values
                is AST.TraitDecl -> it.children().groupBy(AST.Named::name).values
                is AST.EnumDecl -> it.enumEntries.groupBy(AST.Named::name).values
                is AST.ExpressionList -> it.expressions.filterIsInstance<AST.FunctionDecl>()
                    .groupBy(AST.Named::name).values

                is AST.AbstractFunctionDecl -> it.params.groupBy(AST.Named::name).values
                is AST.ForeignFunctionDecl -> it.params.groupBy(AST.Named::name).values
                is AST.FunctionDecl -> it.params.groupBy(AST.Named::name).values

                else -> throw IllegalStateException("Impossible")
            }
        }.filter { it.size > 1 }.map { NonEmptyList.fromListUnsafe(it) }

        duplicates.reportEach { dups ->
            Report(ast.filename, dups.map { it.rlt.position }, Report.Severity.ERROR, SemanticError.Duplicated)
        }
    }
}
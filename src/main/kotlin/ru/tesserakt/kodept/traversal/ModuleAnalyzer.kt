package ru.tesserakt.kodept.traversal

import arrow.core.NonEmptyList
import arrow.core.continuations.eagerEffect
import arrow.core.nonEmptyListOf
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.rlt
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.error.SemanticWarning
import ru.tesserakt.kodept.parser.RLT

val moduleNameAnalyzer = Analyzer { ast ->
    eagerEffect {
        ast.flatten().filterIsInstance<AST.ModuleDecl>()
            .groupBy { it.name }
            .values
            .filter { it.size > 1 }
            .map { NonEmptyList.fromListUnsafe(it) }
            .reportEach { modules ->
                Report(
                    ast.filename,
                    modules.map { it.rlt.id.position },
                    Report.Severity.ERROR,
                    SemanticError.DuplicatedModules(modules.head.name)
                )
            }
    }
}

val moduleUniquenessAnalyzer = Analyzer { ast ->
    eagerEffect {
        val modules = ast.flatten().filterIsInstance<AST.ModuleDecl>().filter { !it.global }.toList()
        if (modules.size != 1) return@eagerEffect
        val head = modules.first()
        val rlt = head.rlt as RLT.Module.Ordinary

        Report(
            ast.filename,
            nonEmptyListOf(rlt.lbrace, rlt.rbrace).map { it.position },
            Report.Severity.WARNING,
            SemanticWarning.NonGlobalSingleModule(head.name)
        ).report()
    }
}
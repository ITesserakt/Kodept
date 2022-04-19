package ru.tesserakt.kodept.analyzer

import ru.tesserakt.kodept.AST
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.visitor.Declaration
import ru.tesserakt.kodept.visitor.DeclarationCollector

class DeclarationAnalyzer : Analyzer {
    private val collector = DeclarationCollector()

    private fun Declaration.mangle(): String = "${parent?.mangle().orEmpty()}-$name"

    private fun List<Declaration>.uniqueVars() = filter { it.decl is AST.VariableDecl }.distinctBy {
        "${it.scope.module}-${it.mangle()}"
    }.toSet()

    override fun analyze(files: Sequence<AST>) =
        files.map { collector.collect(it.root) to it.fileName }.flatMap { (list, file) ->
            (list - list.uniqueVars()).map {
                Report(file,
                    it.decl.coordinates,
                    Report.Severity.ERROR,
                    SemanticError.DuplicatedVariables(it.scope.module, it.name))
            }
        }
}
package ru.tesserakt.kodept.analyzer

import ru.tesserakt.kodept.AST
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.visitor.Declaration
import ru.tesserakt.kodept.visitor.DeclarationCollector
import ru.tesserakt.kodept.visitor.ReferencesCollector

class ReferenceAnalyzer : Analyzer {
    private val collector = DeclarationCollector()
    private val references = ReferencesCollector()

    private fun List<Declaration>.resolve(term: AST.Term, reportSink: () -> Report) = when (term) {
        is AST.TermChain -> TODO()
        is AST.UnresolvedFunctionCall -> TODO()
        is AST.UnresolvedReference -> mapNotNull {
            when (it.decl) {
                is AST.InitializedVar -> null
                else -> null
            }
        }
    }

    override fun analyze(files: Sequence<AST>): Sequence<Report> =
        files.map { collector.collect(it.root) to it.fileName }.flatMap { (list, filename) ->
            TODO()
        }
}
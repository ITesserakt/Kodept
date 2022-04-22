package ru.tesserakt.kodept.analyzer

import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Declaration
import ru.tesserakt.kodept.core.scope
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.visitor.DeclarationCollector
import ru.tesserakt.kodept.visitor.ReferencesCollector

class ReferenceAnalyzer : Analyzer() {
    private val collector = DeclarationCollector()
    private val references = ReferencesCollector()

    private fun Declaration.isDescriptorOf(term: AST.FunctionCall) =
        decl is AST.FunctionDecl
                && term.scope isSubScopeOf decl.scope
                && decl.name == term.name
                && decl.params.count() == decl.params.count()

    private fun Declaration.isDescriptorOf(term: AST.Reference) = decl.scope isSuperScopeOf term.scope
            && when (decl) {
        is AST.InitializedVar -> decl.name == term.name
        is AST.StructDecl -> term.name in decl.alloc.map { it.name }
        is AST.FunctionDecl -> term.name in decl.params.map { it.name }
        else -> false
    }

    private fun Declaration.isDescriptorOf(term: AST.TypeReference) = decl.scope isSuperScopeOf term.scope
            && when (decl) {
        is AST.EnumDecl -> term.name in decl.enumEntries.map { it.name }
        else -> false
    }

    private fun List<Declaration>.resolveByName(term: AST.Term): List<Declaration> = when (term) {
        is AST.TermChain -> TODO()
        is AST.FunctionCall -> filter { it.isDescriptorOf(term) }
        is AST.Reference -> filter { it.isDescriptorOf(term) }
        is AST.TypeReference -> filter { it.isDescriptorOf(term) }
    }

    override fun analyzeIndependently(ast: AST) {
        val declarations = collector.collect(ast.root)
        val terms = references.collect(ast.root)
        collector.collectedReports.report()
        references.collectedReports.report()

        terms.filterIsInstance<AST.Term.Simple>().map { declarations.resolveByName(it) to it }
            .filter { it.first.size != 1 }.reportEach { (descriptors, term) ->
                when (descriptors.size) {
                    0 -> Report(ast.fileName,
                        term.coordinates,
                        Report.Severity.ERROR,
                        SemanticError.UndeclaredUsage(term.name))
                    else -> Report(ast.fileName,
                        term.coordinates,
                        Report.Severity.ERROR,
                        SemanticError.AmbitiousReference(term.name))
                }
            }
    }
}
package ru.tesserakt.kodept.analyzer

import arrow.core.*
import io.arrow.core.mapLeft
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.error.UnrecoverableError
import ru.tesserakt.kodept.visitor.DeclarationCollector
import ru.tesserakt.kodept.visitor.ReferencesCollector
import ru.tesserakt.kodept.visitor.ScopeCollector

private sealed interface SearchContinuation
private object NotFound : SearchContinuation
private object RecurseUp : SearchContinuation

private typealias SearchResultNel<T> = Either<SearchContinuation, Nel<T>>
private typealias ScopeTable = Map<Scope, List<Declaration>>
private typealias ScopedDecls = Pair<Scope, List<Declaration>>

private fun <T> Option<NonEmptyList<T>>.orRecurseFurther(): SearchResultNel<T> = toEither { RecurseUp }
private fun <T> Option<NonEmptyList<T>>.orFail(): SearchResultNel<T> = toEither { NotFound }

private fun List<Declaration>.findAny(vararg predicate: (Declaration) -> Boolean) =
    NonEmptyList.fromList(filter { predicate.any { p -> p(it) } })

private fun <K, V> Map<K, V>.entry(key: K) = get(key)?.let { key to it }

private fun <T : AST.Term> ScopeTable.expandRecursion(
    term: T, scope: Scope = term.scope, using: ScopedDecls.(T) -> SearchResultNel<Declaration>,
): Either<NotFound, NonEmptyList<Declaration>> = entry(scope)
    .rightIfNotNull { RecurseUp }
    .flatMap { it.using(term) }
    .handleErrorWith {
        when (it) {
            NotFound -> NotFound.left()
            RecurseUp -> (scope as? Scope.Inner<*>)?.parent
                .rightIfNotNull { NotFound }
                .flatMap { parent -> expandRecursion(term, parent, using) }
        }
    }

class ReferenceAnalyzer : Analyzer() {
    private val collector = DeclarationCollector()
    private val references = ReferencesCollector()
    private val scopeCollector = ScopeCollector()

    private fun ScopedDecls.findReference(term: AST.Reference): SearchResultNel<Declaration> =
        when (first) {
            is Scope.Global -> None.orFail()
            is Scope.Local -> second.findAny(
                { it.decl is AST.InitializedVar && it.name == term.name },
                { it.decl is AST.VariableDecl && it.name == term.name },
                { it.decl is AST.FunctionDecl && term.name in it.decl.params.map(AST.NamedDecl::name) }
            ).orRecurseFurther()
            is Scope.Object -> second.findAny(
                { it.decl is AST.StructDecl && term.name in it.decl.alloc.map(AST.NamedDecl::name) },
                { it.decl is AST.FunctionDecl && term.name in it.decl.params.map(AST.NamedDecl::name) }
            ).orRecurseFurther()
        }

    private fun ScopedDecls.findFunction(term: AST.FunctionCall) = when (first) {
        is Scope.Global -> second.findAny(
            { it.decl is AST.FunctionDecl }
        ).orFail()
        is Scope.Inner<*> -> second.findAny(
            { it.decl is AST.FunctionDecl }
        ).orRecurseFurther()
    }

    private fun ScopedDecls.findType(term: AST.TypeReference) =
        when (first) {
            is Scope.Inner<*> -> None.orRecurseFurther()
            is Scope.Global -> second.findAny(
                { it.decl is AST.TopLevelDecl && it.decl !is AST.FunctionDecl && term.name == it.name }
            ).orFail()
        }

    private fun ScopeTable.resolveByName(term: AST.Term) = when (term) {
        is AST.TermChain -> TODO()
        is AST.FunctionCall -> expandRecursion(term) { findFunction(it) }
        is AST.Reference -> expandRecursion(term) { findReference(it) }
        is AST.TypeReference -> expandRecursion(term) { findType(it) }
    }.fold({ emptyList() }, { it.all })

    override fun analyzeIndependently(ast: AST) {
        val (declarations, terms, scopes) = Either.catch {
            (collector.collect(ast.root) to references.collect(ast.root)) + scopeCollector.collect(ast.root)
        }.getOrHandle {
            collector.collectedReports.report()
            references.collectedReports.report()
            scopeCollector.collectedReports.report()
            throw UnrecoverableError(this)
        }
        val table = scopes.associateWith { declarations.filter { declaration -> declaration.decl.scope == it } }

        val (found, erroneous) = terms
            .filterIsInstance<AST.Term.Simple>()
            .map { table.resolveByName(it) to it }
            .partition { it.first.size == 1 }
            .mapLeft { it.map { pair -> pair.mapLeft(List<Declaration>::first) } }

        found.forEach { (descriptor, term) ->
            term.metadata += MetadataStore.Key.TermDescriptor(descriptor.decl)
        }

        erroneous.reportEach { (descriptors, term) ->
            when (descriptors.size) {
                0 -> Report(ast.fileName,
                    term.metadata.retrieveRequired<MetadataStore.Key.RLTReference>().value.position.nel(),
                    Report.Severity.ERROR,
                    SemanticError.UndeclaredUsage("TODO"))
                else -> Report(ast.fileName,
                    term.metadata.retrieveRequired<MetadataStore.Key.RLTReference>().value.position.nel(),
                    Report.Severity.ERROR,
                    SemanticError.AmbitiousReference("TODO"))
            }
        }

        if (erroneous.isNotEmpty())
            throw UnrecoverableError(this)
    }
}
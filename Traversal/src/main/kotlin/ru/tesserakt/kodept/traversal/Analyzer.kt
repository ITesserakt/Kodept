package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Tree
import ru.tesserakt.kodept.error.ReportCollector

abstract class Analyzer : Depended() {
    protected open val walkMode = Tree.SearchMode.LevelOrder
    protected abstract fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit>

    fun ReportCollector.analyzeWithCaching(ast: AST): EagerEffect<UnrecoverableError, Unit> {
        treeCache.computeIfAbsent(ast) { it.flatten(walkMode).toList() }
        return analyze(ast)
    }

    protected fun AST.fastFlatten(mode: Tree.SearchMode = walkMode, filtering: (AST.Node) -> Boolean = { true }) =
        treeCache[this]?.filter(filtering)?.asSequence() ?: flatten(mode).filter(filtering)

    companion object {
        private var treeCache = hashMapOf<AST, List<AST.Node>>()
    }
}
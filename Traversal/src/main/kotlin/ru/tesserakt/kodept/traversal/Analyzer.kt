package ru.tesserakt.kodept.traversal

import arrow.core.raise.EagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Tree
import ru.tesserakt.kodept.error.ReportCollector

abstract class Analyzer : Depended() {
    protected open val walkMode = Tree.SearchMode.LevelOrder
    protected abstract fun ReportCollector.analyze(ast: AST): EagerEffect<UnrecoverableError, Unit>

    context (ReportCollector)
    fun analyzeWithCaching(ast: AST): EagerEffect<UnrecoverableError, Unit> {
        return analyze(ast)
    }

    protected fun AST.fastFlatten(mode: Tree.SearchMode = walkMode, filtering: (AST.Node) -> Boolean = { true }) =
        flatten(mode).filter(filtering)
}
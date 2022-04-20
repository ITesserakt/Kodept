package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.error.ReportCollector

abstract class NodeCollector<T> : ReportCollector() {
    protected abstract val underlyingVisitor: NodeProcessor<*>

    abstract fun collect(start: AST.Node): T
}
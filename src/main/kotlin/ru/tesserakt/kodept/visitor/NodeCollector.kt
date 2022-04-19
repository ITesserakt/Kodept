package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.AST

abstract class NodeCollector<T> {
    protected abstract val underlyingVisitor: NodeProcessor<*>

    abstract fun collect(start: AST.Node): T
}
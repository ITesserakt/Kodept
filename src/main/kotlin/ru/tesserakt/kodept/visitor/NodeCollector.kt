package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.parser.AST

abstract class NodeCollector<T, R> {
    protected abstract val underlyingVisitor: NodeProcessor<R>

    abstract fun collect(start: AST.Node): T
}
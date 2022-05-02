package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Scope
import ru.tesserakt.kodept.core.scope

class ScopeCollector : NodeCollector<Set<Scope>>() {
    override val underlyingVisitor = object : NodeProcessor<Scope>() {
        override fun default(node: AST.Node): Scope = node.scope
    }

    override fun collect(start: AST.Node) =
        start.acceptRecursively(underlyingVisitor).toSet()
}
package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.AST

class ReferencesCollector : NodeCollector<List<AST.Term>>() {
    override val underlyingVisitor = object : NodeProcessor<AST.Term?>() {
        override fun default(node: AST.Node) = null

        override fun visit(node: AST.UnresolvedReference): AST.Term = node
        override fun visit(node: AST.UnresolvedFunctionCall): AST.Term = node
        override fun visit(node: AST.TermChain): AST.Term = node
    }

    override fun collect(start: AST.Node): List<AST.Term> = start.acceptRecursively(underlyingVisitor).filterNotNull()
}
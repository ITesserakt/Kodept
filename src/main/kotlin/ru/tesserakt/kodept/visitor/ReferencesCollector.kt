package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.core.AST

class ReferencesCollector : NodeCollector<List<AST.Term>>() {
    override val underlyingVisitor = object : NodeProcessor<AST.Term?>() {
        override fun default(node: AST.Node) = null

        override fun visit(node: AST.Reference) = node
        override fun visit(node: AST.FunctionCall) = node
        override fun visit(node: AST.TermChain) = node
        override fun visit(node: AST.TypeReference) = node
    }

    override fun collect(start: AST.Node): List<AST.Term> = start.acceptRecursively(underlyingVisitor).filterNotNull()
}
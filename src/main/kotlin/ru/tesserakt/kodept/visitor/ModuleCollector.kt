package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.parser.AST

class ModuleCollector : NodeCollector<List<AST.ModuleDecl>>() {
    override val underlyingVisitor = object : NodeProcessor<AST.ModuleDecl?>() {
        override fun default(node: AST.Node): AST.ModuleDecl? = null
        override fun visit(node: AST.ModuleDecl) = node
    }

    override fun collect(start: AST.Node) = start.acceptRecursively(underlyingVisitor).filterNotNull()
}
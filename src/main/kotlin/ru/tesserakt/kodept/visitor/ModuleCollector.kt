package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.parser.AST

class ModuleCollector : NodeCollector<List<String>>() {
    private var modules = mutableListOf<String>()

    override val underlyingVisitor = object : NodeVisitor() {
        override fun visit(node: AST.ModuleDecl) {
            modules += node.name
        }
    }

    override fun collect(start: AST.Node): List<String> {
        modules.clear()
        start.acceptRecursively(underlyingVisitor)
        return modules
    }
}
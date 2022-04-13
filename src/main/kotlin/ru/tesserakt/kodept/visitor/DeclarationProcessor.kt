package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.parser.AST
import kotlin.reflect.KProperty0

data class Declaration(val decl: AST.Node, val parent: Declaration?, val name: String, val module: String)

class DeclarationProcessor : NodeCollector<List<Declaration>, List<Declaration>>() {
    private var declParent: Declaration? = null
    private var currentModule: String = ""

    private fun <T : AST.Node> T.traverseWithChildren(
        nameProperty: KProperty0<String>,
        children: KProperty0<List<AST.Node>>,
    ): List<Declaration> {
        val saved = declParent
        declParent = Declaration(this, declParent, nameProperty(), currentModule)
        return (listOfNotNull(declParent) + children().flatMap { it.accept(underlyingVisitor) }).also {
            declParent = saved
        }
    }

    private fun <T : AST.Node> T.traverseWithChild(
        nameProperty: KProperty0<String>,
        child: KProperty0<AST.Node>? = null,
    ): List<Declaration> {
        val saved = declParent
        declParent = Declaration(this, declParent, nameProperty(), currentModule)
        return (listOfNotNull(declParent) + child?.invoke()?.accept(underlyingVisitor).orEmpty()).also {
            declParent = saved
        }
    }

    override val underlyingVisitor: NodeProcessor<List<Declaration>> = object : NodeProcessor<List<Declaration>>() {
        override fun default(node: AST.Node): List<Declaration> = emptyList()

        override fun visit(node: AST.WhileExpr): List<Declaration> = node.body.accept(this)

        override fun visit(node: AST.IfExpr): List<Declaration> =
            node.body.accept(this) + node.elifs.flatMap { it.accept(this) } + node.el?.accept(this).orEmpty()

        override fun visit(node: AST.ExpressionList): List<Declaration> = node.expressions.flatMap { it.accept(this) }

        override fun visit(node: AST.IfExpr.ElifExpr): List<Declaration> = node.body.accept(this)

        override fun visit(node: AST.IfExpr.ElseExpr): List<Declaration> = node.body.accept(this)

        override fun visit(node: AST.FileDecl) = node.modules.flatMap { it.accept(this) }

        override fun visit(node: AST.Assignment): List<Declaration> = node.left.accept(this) + node.right.accept(this)

        override fun visit(node: AST.ModuleDecl): List<Declaration> {
            currentModule = node.name
            return node.rest.flatMap { it.accept(this) }
        }

        override fun visit(node: AST.FunctionDecl) = node.traverseWithChild(node::name, node::rest)
        override fun visit(node: AST.InitializedVar) = node.traverseWithChild(node::name, node::expr)
        override fun visit(node: AST.VariableDecl) = node.traverseWithChild(node::name)
        override fun visit(node: AST.EnumDecl.Entry) = node.traverseWithChild(node::name)
        override fun visit(node: AST.EnumDecl) = node.traverseWithChildren(node::name, node::enumEntries)
        override fun visit(node: AST.StructDecl) = node.traverseWithChildren(node::name, node::rest)
        override fun visit(node: AST.TraitDecl) = node.traverseWithChildren(node::name, node::rest)
    }

    override fun collect(start: AST.Node): List<Declaration> {
        declParent = null
        currentModule = ""
        return start.accept(underlyingVisitor)
    }
}
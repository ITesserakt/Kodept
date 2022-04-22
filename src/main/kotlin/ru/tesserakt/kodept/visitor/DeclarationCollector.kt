package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Declaration
import kotlin.reflect.KProperty0

class DeclarationCollector : NodeCollector<List<Declaration>>() {
    private var declParent: Declaration? = null

    private inline fun <T> Declaration.restoreParent(block: Declaration.() -> T): T {
        val saved = declParent
        declParent = this
        return block().also { declParent = saved }
    }

    private fun <T : AST.Node> T.traverseWithChildren(
        nameProperty: KProperty0<String>,
        children: KProperty0<List<AST.Node>>,
    ) = Declaration(this, declParent, nameProperty()).restoreParent {
        listOf(this) + children().flatMap { it.accept(underlyingVisitor) }
    }

    private fun <T : AST.Node> T.traverseWithChild(
        nameProperty: KProperty0<String>,
        child: KProperty0<AST.Node>? = null,
    ) = Declaration(this, declParent, nameProperty()).restoreParent {
        listOf(this) + child?.invoke()?.accept(underlyingVisitor).orEmpty()
    }

    override val underlyingVisitor: NodeProcessor<List<Declaration>> = object : NodeProcessor<List<Declaration>>() {
        override fun default(node: AST.Node): List<Declaration> = emptyList()

        override fun visit(node: AST.WhileExpr) = node.body.accept(this)

        override fun visit(node: AST.IfExpr) =
            node.body.accept(this) + node.elifs.flatMap { it.accept(this) } + node.el?.accept(this).orEmpty()

        override fun visit(node: AST.ExpressionList) = node.expressions.flatMap { it.accept(this) }

        override fun visit(node: AST.IfExpr.ElifExpr) = node.body.accept(this)

        override fun visit(node: AST.IfExpr.ElseExpr) = node.body.accept(this)

        override fun visit(node: AST.FileDecl) = node.modules.all.flatMap { it.accept(this) }

        override fun visit(node: AST.Assignment): List<Declaration> = node.left.accept(this) + node.right.accept(this)

        override fun visit(node: AST.FunctionDecl) = node.traverseWithChild(node::name, node::rest)
        override fun visit(node: AST.InitializedVar) = node.traverseWithChild(node::name, node::expr)
        override fun visit(node: AST.VariableDecl) = node.traverseWithChild(node::name)

        override fun visit(node: AST.EnumDecl) = node.traverseWithChildren(node::name, node::enumEntries)
        override fun visit(node: AST.ModuleDecl) = node.traverseWithChildren(node::name, node::rest)
        override fun visit(node: AST.StructDecl) = node.traverseWithChildren(node::name, node::rest)
        override fun visit(node: AST.TraitDecl) = node.traverseWithChildren(node::name, node::rest)
    }

    override fun collect(start: AST.Node): List<Declaration> {
        declParent = null
        return start.accept(underlyingVisitor)
    }
}
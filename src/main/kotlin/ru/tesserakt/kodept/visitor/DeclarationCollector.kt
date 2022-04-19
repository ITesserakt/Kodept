package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.AST
import ru.tesserakt.kodept.visitor.Scope.*
import kotlin.reflect.KProperty0

class DeclarationCollector : NodeCollector<List<Declaration>>() {
    private var declParent: Declaration? = null
    private var currentScope: Scope = Global("")

    private fun Scope.asInner() = this as Inner<*>

    private inline fun <T> Declaration.restoreParent(block: Declaration.() -> T): T {
        val saved = declParent
        declParent = this
        return block().also { declParent = saved }
    }

    private inline fun <T, S : Scope> scope(ctor: (Scope) -> S, block: (S) -> T): T {
        val scope = ctor(currentScope)
        val saved = currentScope
        currentScope = scope
        return block(scope).also { currentScope = saved }
    }

    private inline fun <T> localScope(block: (Local) -> T): T = scope({ Local(it.asInner()) }, block)

    private inline fun <T> innerScope(block: (Inner<*>) -> T): T =
        scope({ (it as? Global)?.let(::Object) ?: Local(it.asInner()) }, block)

    private inline fun <T> objectScope(block: (Object) -> T) = scope({ Object(it as Global) }, block)

    private fun <T : AST.Node> T.traverseWithChildren(
        nameProperty: KProperty0<String>,
        children: KProperty0<List<AST.Node>>,
    ) = Declaration(this, declParent, nameProperty(), currentScope).restoreParent {
        listOf(this) + children().flatMap { it.accept(underlyingVisitor) }
    }

    private fun <T : AST.Node> T.traverseWithChild(
        nameProperty: KProperty0<String>,
        child: KProperty0<AST.Node>? = null,
    ) = Declaration(this, declParent, nameProperty(), currentScope).restoreParent {
        listOf(this) + child?.invoke()?.accept(underlyingVisitor).orEmpty()
    }

    override val underlyingVisitor: NodeProcessor<List<Declaration>> = object : NodeProcessor<List<Declaration>>() {
        override fun default(node: AST.Node): List<Declaration> = emptyList()

        override fun visit(node: AST.WhileExpr) = localScope {
            node.body.accept(this)
        }

        override fun visit(node: AST.IfExpr) =
            localScope { node.body.accept(this) } +
                    node.elifs.flatMap { it.accept(this) } +
                    node.el?.accept(this).orEmpty()

        override fun visit(node: AST.ExpressionList) = localScope { node.expressions.flatMap { it.accept(this) } }

        override fun visit(node: AST.IfExpr.ElifExpr) = localScope { node.body.accept(this) }

        override fun visit(node: AST.IfExpr.ElseExpr) = localScope { node.body.accept(this) }

        override fun visit(node: AST.FileDecl) = node.modules.flatMap { it.accept(this) }

        override fun visit(node: AST.Assignment): List<Declaration> = node.left.accept(this) + node.right.accept(this)

        override fun visit(node: AST.ModuleDecl): List<Declaration> {
            currentScope = Global(node.name)
            return node.rest.flatMap { it.accept(this) }
        }

        override fun visit(node: AST.FunctionDecl) = innerScope { node.traverseWithChild(node::name, node::rest) }
        override fun visit(node: AST.InitializedVar) = node.traverseWithChild(node::name, node::expr)
        override fun visit(node: AST.VariableDecl) = node.traverseWithChild(node::name)
        override fun visit(node: AST.EnumDecl.Entry) = node.traverseWithChild(node::name)
        override fun visit(node: AST.EnumDecl) =
            objectScope { node.traverseWithChildren(node::name, node::enumEntries) }

        override fun visit(node: AST.StructDecl) = objectScope { node.traverseWithChildren(node::name, node::rest) }
        override fun visit(node: AST.TraitDecl) = objectScope { node.traverseWithChildren(node::name, node::rest) }
    }

    override fun collect(start: AST.Node): List<Declaration> {
        declParent = null
        currentScope = Global("")
        return start.accept(underlyingVisitor)
    }
}
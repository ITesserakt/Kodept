package ru.tesserakt.kodept.transformer

import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Scope
import ru.tesserakt.kodept.core.appendMetadata
import ru.tesserakt.kodept.core.toKey
import java.util.*

class ASTScopeTagger : Transformer() {
    private var currentScope: Scope = Scope.Global("")

    private fun Scope.asInner() = this as Scope.Inner<*>

    private inline fun <T, S : Scope> scope(ctor: (Scope) -> S, block: (S) -> T): T {
        val scope = ctor(currentScope)
        val saved = currentScope
        currentScope = scope
        return block(scope).also { currentScope = saved }
    }

    private inline fun <T> localScope(block: (Scope.Local) -> T): T =
        scope({ Scope.Local(it.asInner(), UUID.randomUUID()) }, block)

    private inline fun <T> innerScope(block: (Scope.Inner<*>) -> T): T = scope({ scope ->
        (scope as? Scope.Global)?.let { Scope.Object(it, UUID.randomUUID()) } ?: Scope.Local(scope.asInner(),
            UUID.randomUUID())
    }, block)

    private inline fun <T> objectScope(block: (Scope.Object) -> T) =
        scope({ Scope.Object(it as Scope.Global, UUID.randomUUID()) }, block)

    private val self get() = this

    override fun visit(node: AST.ModuleDecl): AST.Node {
        currentScope = Scope.Global(node.name)
        return node.copy(metadata = node.appendMetadata(currentScope.toKey()),
            rest = node.rest.map { it.acceptTransform(self) })
    }

    override fun visit(node: AST.WhileExpr) = node.copy(
        metadata = node.appendMetadata(currentScope.toKey()),
        condition = localScope { node.condition.acceptTransform(self) },
        body = localScope { node.body.acceptTransform(self) })

    override fun visit(node: AST.IfExpr) = node.copy(
        metadata = node.appendMetadata(currentScope.toKey()),
        condition = localScope { node.condition.acceptTransform(self) },
        body = localScope { node.body.acceptTransform(self) },
        elifs = node.elifs.map { it.acceptTransform(self) },
        el = node.el?.acceptTransform(self))

    override fun visit(node: AST.ExpressionList) = node.copy(
        metadata = node.appendMetadata(currentScope.toKey()),
        expressions = localScope { node.expressions.map { it.acceptTransform(self) } })

    override fun visit(node: AST.FunctionDecl) = innerScope {
        node.copy(
            metadata = node.appendMetadata(currentScope.toKey()),
            params = node.params.map { it.acceptTransform(self) },
            returns = node.returns?.acceptTransform(self),
            rest = node.rest.acceptTransform(self))
    }

    override fun visit(node: AST.EnumDecl) = objectScope {
        node.copy(
            metadata = node.appendMetadata(currentScope.toKey()),
            enumEntries = node.enumEntries.map { it.acceptTransform(self) })
    }

    override fun visit(node: AST.StructDecl) = objectScope {
        node.copy(
            metadata = node.appendMetadata(currentScope.toKey()),
            alloc = node.alloc.map { it.acceptTransform(self) },
            rest = node.rest.map { it.acceptTransform(self) })
    }

    override fun visit(node: AST.TraitDecl) = objectScope {
        node.copy(
            metadata = node.appendMetadata(currentScope.toKey()),
            rest = node.rest.map { it.acceptTransform(self) })
    }

    override fun visit(node: AST.IfExpr.ElifExpr) = node.copy(
        metadata = node.appendMetadata(currentScope.toKey()),
        condition = localScope { node.condition.acceptTransform(self) },
        body = localScope { node.body.acceptTransform(self) })

    override fun visit(node: AST.IfExpr.ElseExpr) = node.copy(
        metadata = node.appendMetadata(currentScope.toKey()),
        body = localScope { node.body.acceptTransform(self) })

    override fun visit(node: AST.CharLiteral) = node.copy(metadata = node.appendMetadata(currentScope.toKey()))
    override fun visit(node: AST.BinaryLiteral) = node.copy(metadata = node.appendMetadata(currentScope.toKey()))
    override fun visit(node: AST.DecimalLiteral) = node.copy(metadata = node.appendMetadata(currentScope.toKey()))
    override fun visit(node: AST.FloatingLiteral) = node.copy(metadata = node.appendMetadata(currentScope.toKey()))
    override fun visit(node: AST.HexLiteral) = node.copy(metadata = node.appendMetadata(currentScope.toKey()))
    override fun visit(node: AST.OctalLiteral) = node.copy(metadata = node.appendMetadata(currentScope.toKey()))
    override fun visit(node: AST.StringLiteral) = node.copy(metadata = node.appendMetadata(currentScope.toKey()))
    override fun visit(node: AST.Assignment) = node.copy(metadata = node.appendMetadata(currentScope.toKey()),
        left = node.left.acceptTransform(self),
        right = node.right.acceptTransform(self))

    override fun visit(node: AST.Binary) = node.copy(metadata = node.appendMetadata(currentScope.toKey()),
        left = node.left.acceptTransform(self),
        right = node.right.acceptTransform(self))

    override fun visit(node: AST.Comparison) = node.copy(metadata = node.appendMetadata(currentScope.toKey()),
        left = node.left.acceptTransform(self),
        right = node.right.acceptTransform(self))

    override fun visit(node: AST.Elvis) = node.copy(metadata = node.appendMetadata(currentScope.toKey()),
        left = node.left.acceptTransform(self),
        right = node.right.acceptTransform(self))

    override fun visit(node: AST.Logical) = node.copy(metadata = node.appendMetadata(currentScope.toKey()),
        left = node.left.acceptTransform(self),
        right = node.right.acceptTransform(self))

    override fun visit(node: AST.Mathematical) = node.copy(metadata = node.appendMetadata(currentScope.toKey()),
        left = node.left.acceptTransform(self),
        right = node.right.acceptTransform(self))

    override fun visit(node: AST.Absolution) =
        node.copy(metadata = node.appendMetadata(currentScope.toKey()), expr = node.expr.acceptTransform(self))

    override fun visit(node: AST.BitInversion) =
        node.copy(metadata = node.appendMetadata(currentScope.toKey()), expr = node.expr.acceptTransform(self))

    override fun visit(node: AST.Inversion) =
        node.copy(metadata = node.appendMetadata(currentScope.toKey()), expr = node.expr.acceptTransform(self))

    override fun visit(node: AST.Negation) =
        node.copy(metadata = node.appendMetadata(currentScope.toKey()), expr = node.expr.acceptTransform(self))

    override fun visit(node: AST.TermChain) = node.copy(metadata = node.appendMetadata(currentScope.toKey()),
        terms = node.terms.map { it.acceptTransform(self) })

    override fun visit(node: AST.FunctionCall) =
        node.copy(metadata = node.appendMetadata(currentScope.toKey()),
            reference = node.reference.acceptTransform(self),
            params = node.params.map { it.acceptTransform(self) })

    override fun visit(node: AST.Reference) = node.copy(metadata = node.appendMetadata(currentScope.toKey()))
    override fun visit(node: AST.TypeExpression) = node.copy(metadata = node.appendMetadata(currentScope.toKey()))
    override fun visit(node: AST.FunctionDecl.Parameter) = node.copy(
        metadata = node.appendMetadata(currentScope.toKey()),
        type = node.type.acceptTransform(self))

    override fun visit(node: AST.InitializedVar) = node.copy(
        metadata = node.appendMetadata(currentScope.toKey()),
        decl = node.decl.acceptTransform(self),
        expr = node.expr.acceptTransform(self))

    override fun visit(node: AST.VariableDecl) = node.copy(
        metadata = node.appendMetadata(currentScope.toKey()),
        type = node.type?.acceptTransform(self))

    override fun visit(node: AST.FileDecl) = node.copy(
        metadata = node.appendMetadata(currentScope.toKey()),
        modules = node.modules.map { it.acceptTransform(self) })

    override fun visit(node: AST.EnumDecl.Entry) = node.copy(metadata = node.appendMetadata(currentScope.toKey()))
    override fun visit(node: AST.StructDecl.Parameter) = node.copy(
        metadata = node.appendMetadata(currentScope.toKey()),
        type = node.type.acceptTransform(self))

    override fun visit(node: AST.TypeReference) = node.copy(type = node.type.acceptTransform(self))
}
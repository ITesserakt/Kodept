@file:Suppress("unused")

package ru.tesserakt.kodept.visitor

import arrow.core.NonEmptyList
import ru.tesserakt.kodept.parser.AST.*

abstract class NodeProcessor<T> {
    open fun default(node: Node): T = throw IllegalArgumentException("Override either this method or all visit methods")

    open fun visit(node: WhileExpr): T = default(node)
    open fun visit(node: IfExpr): T = default(node)
    open fun visit(node: ExpressionList): T = default(node)
    open fun visit(node: CharLiteral): T = default(node)
    open fun visit(node: BinaryLiteral): T = default(node)
    open fun visit(node: DecimalLiteral): T = default(node)
    open fun visit(node: FloatingLiteral): T = default(node)
    open fun visit(node: HexLiteral): T = default(node)
    open fun visit(node: OctalLiteral): T = default(node)
    open fun visit(node: StringLiteral): T = default(node)
    open fun visit(node: Assignment): T = default(node)
    open fun visit(node: Binary): T = default(node)
    open fun visit(node: Comparison): T = default(node)
    open fun visit(node: Elvis): T = default(node)
    open fun visit(node: Logical): T = default(node)
    open fun visit(node: Mathematical): T = default(node)
    open fun visit(node: Absolution): T = default(node)
    open fun visit(node: BitInversion): T = default(node)
    open fun visit(node: Inversion): T = default(node)
    open fun visit(node: Negation): T = default(node)
    open fun visit(node: TermChain): T = default(node)
    open fun visit(node: UnresolvedFunctionCall): T = default(node)
    open fun visit(node: UnresolvedReference): T = default(node)
    open fun visit(node: TypeExpression): T = default(node)
    open fun visit(node: FunctionDecl): T = default(node)
    open fun visit(node: FunctionDecl.Parameter): T = default(node)
    open fun visit(node: InitializedVar): T = default(node)
    open fun visit(node: VariableDecl): T = default(node)
    open fun visit(node: FileDecl): T = default(node)
    open fun visit(node: EnumDecl): T = default(node)
    open fun visit(node: EnumDecl.Entry): T = default(node)
    open fun visit(node: ModuleDecl): T = default(node)
    open fun visit(node: StructDecl): T = default(node)
    open fun visit(node: StructDecl.Parameter): T = default(node)
    open fun visit(node: TraitDecl): T = default(node)
    open fun visit(node: IfExpr.ElifExpr): T = default(node)
    open fun visit(node: IfExpr.ElseExpr): T = default(node)
}

abstract class NodeVisitor : NodeProcessor<Unit>() {
    final override fun default(node: Node) {}
}

abstract class IntermediateNodeProcessor<T> : NodeProcessor<List<T>>() {
    final override fun default(node: Node) = listOfNotNull(
        (node as? CodeFlowExpr)?.let(::visit),
        (node as? Term)?.let(::visit),
        (node as? Operation)?.let(::visit),
        (node as? Literal)?.let(::visit),
        (node as? Expression)?.let(::visit),
        (node as? ObjectDecl)?.let(::visit),
        (node as? CallableDecl)?.let(::visit),
        (node as? TypedDecl)?.let(::visit),
        (node as? NamedDecl)?.let(::visit),
        (node as? BlockLevelDecl)?.let(::visit),
        (node as? ObjectLevelDecl)?.let(::visit),
        (node as? TopLevelDecl)?.let(::visit),
        (node as? Leaf)?.let(::visit),
        visit(node)
    )

    abstract fun visit(node: Node): T
    abstract fun visit(node: Leaf): T
    abstract fun visit(node: TopLevelDecl): T
    abstract fun visit(node: ObjectLevelDecl): T
    abstract fun visit(node: BlockLevelDecl): T
    abstract fun visit(node: NamedDecl): T
    abstract fun visit(node: TypedDecl): T
    abstract fun visit(node: CallableDecl): T
    abstract fun visit(node: ObjectDecl): T
    abstract fun visit(node: Expression): T
    abstract fun visit(node: Literal): T
    abstract fun visit(node: Operation): T
    abstract fun visit(node: Term): T
    abstract fun visit(node: CodeFlowExpr): T
}

abstract class IntermediateNodeVisitor : IntermediateNodeProcessor<Unit>() {
    override fun visit(node: Node) {}
    override fun visit(node: Leaf) {}
    override fun visit(node: TopLevelDecl) {}
    override fun visit(node: ObjectLevelDecl) {}
    override fun visit(node: BlockLevelDecl) {}
    override fun visit(node: NamedDecl) {}
    override fun visit(node: TypedDecl) {}
    override fun visit(node: CallableDecl) {}
    override fun visit(node: ObjectDecl) {}
    override fun visit(node: Expression) {}
    override fun visit(node: Literal) {}
    override fun visit(node: Operation) {}
    override fun visit(node: Term) {}
    override fun visit(node: CodeFlowExpr) {}
}

typealias NodeChecker = NodeProcessor<Boolean>

interface Acceptable {
    fun <T> accept(visitor: NodeProcessor<T>): T

    fun <T> acceptRecursively(visitor: NodeProcessor<T>): NonEmptyList<T>
}
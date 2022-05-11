@file:Suppress("unused")

package ru.tesserakt.kodept.visitor

import arrow.core.NonEmptyList
import arrow.core.nonEmptyListOf
import ru.tesserakt.kodept.core.AST.*
import ru.tesserakt.kodept.error.ReportCollector

abstract class NodeProcessor<T> : ReportCollector() {
    open fun default(node: Node): T = throw IllegalArgumentException("Override either this method or all visit methods")

    open fun visit(node: WhileExpr) = default(node)
    open fun visit(node: IfExpr) = default(node)
    open fun visit(node: ExpressionList) = default(node)
    open fun visit(node: CharLiteral) = default(node)
    open fun visit(node: BinaryLiteral) = default(node)
    open fun visit(node: DecimalLiteral) = default(node)
    open fun visit(node: FloatingLiteral) = default(node)
    open fun visit(node: HexLiteral) = default(node)
    open fun visit(node: OctalLiteral) = default(node)
    open fun visit(node: StringLiteral) = default(node)
    open fun visit(node: Assignment) = default(node)
    open fun visit(node: Binary) = default(node)
    open fun visit(node: Comparison) = default(node)
    open fun visit(node: Elvis) = default(node)
    open fun visit(node: Logical) = default(node)
    open fun visit(node: Mathematical) = default(node)
    open fun visit(node: Absolution) = default(node)
    open fun visit(node: BitInversion) = default(node)
    open fun visit(node: Inversion) = default(node)
    open fun visit(node: Negation) = default(node)
    open fun visit(node: TermChain) = default(node)
    open fun visit(node: FunctionCall) = default(node)
    open fun visit(node: Reference) = default(node)
    open fun visit(node: TypeExpression) = default(node)
    open fun visit(node: FunctionDecl) = default(node)
    open fun visit(node: Parameter) = default(node)
    open fun visit(node: InitializedVar) = default(node)
    open fun visit(node: VariableDecl) = default(node)
    open fun visit(node: FileDecl) = default(node)
    open fun visit(node: EnumDecl) = default(node)
    open fun visit(node: EnumDecl.Entry) = default(node)
    open fun visit(node: ModuleDecl) = default(node)
    open fun visit(node: StructDecl) = default(node)
    open fun visit(node: TraitDecl) = default(node)
    open fun visit(node: IfExpr.ElifExpr) = default(node)
    open fun visit(node: IfExpr.ElseExpr) = default(node)
    open fun visit(node: TypeReference) = default(node)
    open fun visit(node: TupleLiteral) = default(node)
    open fun visit(node: InferredParameter) = default(node)
    open fun visit(node: AbstractFunctionDecl) = default(node)
}

abstract class NodeVisitor : NodeProcessor<Unit>() {
    override fun default(node: Node) = Unit
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

fun <T> Node.accept(visitor: NodeProcessor<T>) = when (this) {
    is IfExpr -> visitor.visit(this)
    is WhileExpr -> visitor.visit(this)
    is ExpressionList -> visitor.visit(this)
    is CharLiteral -> visitor.visit(this)
    is BinaryLiteral -> visitor.visit(this)
    is DecimalLiteral -> visitor.visit(this)
    is FloatingLiteral -> visitor.visit(this)
    is HexLiteral -> visitor.visit(this)
    is OctalLiteral -> visitor.visit(this)
    is StringLiteral -> visitor.visit(this)
    is Assignment -> visitor.visit(this)
    is Binary -> visitor.visit(this)
    is Comparison -> visitor.visit(this)
    is Elvis -> visitor.visit(this)
    is Logical -> visitor.visit(this)
    is Mathematical -> visitor.visit(this)
    is Absolution -> visitor.visit(this)
    is BitInversion -> visitor.visit(this)
    is Inversion -> visitor.visit(this)
    is Negation -> visitor.visit(this)
    is TermChain -> visitor.visit(this)
    is FunctionCall -> visitor.visit(this)
    is Reference -> visitor.visit(this)
    is TypeExpression -> visitor.visit(this)
    is FunctionDecl -> visitor.visit(this)
    is InitializedVar -> visitor.visit(this)
    is VariableDecl -> visitor.visit(this)
    is FileDecl -> visitor.visit(this)
    is EnumDecl.Entry -> visitor.visit(this)
    is EnumDecl -> visitor.visit(this)
    is ModuleDecl -> visitor.visit(this)
    is Parameter -> visitor.visit(this)
    is StructDecl -> visitor.visit(this)
    is TraitDecl -> visitor.visit(this)
    is IfExpr.ElifExpr -> visitor.visit(this)
    is IfExpr.ElseExpr -> visitor.visit(this)
    is TypeReference -> visitor.visit(this)
    is TupleLiteral -> visitor.visit(this)
    is InferredParameter -> visitor.visit(this)
    is AbstractFunctionDecl -> visitor.visit(this)
}

fun <T> Node.acceptRecursively(visitor: NodeProcessor<T>): NonEmptyList<T> =
    nonEmptyListOf(accept(visitor)) + children.flatMap { it.acceptRecursively(visitor) }
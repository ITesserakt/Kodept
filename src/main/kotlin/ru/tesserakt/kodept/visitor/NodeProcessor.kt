@file:Suppress("unused")

package ru.tesserakt.kodept.visitor

import arrow.core.NonEmptyList
import ru.tesserakt.kodept.parser.AST.*

interface NodeProcessor<T> {
    fun visit(node: WhileExpr): T
    fun visit(node: IfExpr): T
    fun visit(node: ExpressionList): T
    fun visit(node: CharLiteral): T
    fun visit(node: BinaryLiteral): T
    fun visit(node: DecimalLiteral): T
    fun visit(node: FloatingLiteral): T
    fun visit(node: HexLiteral): T
    fun visit(node: OctalLiteral): T
    fun visit(node: StringLiteral): T
    fun visit(node: Assignment): T
    fun visit(node: Binary): T
    fun visit(node: Comparison): T
    fun visit(node: Elvis): T
    fun visit(node: Logical): T
    fun visit(node: Mathematical): T
    fun visit(node: Absolution): T
    fun visit(node: BitInversion): T
    fun visit(node: Inversion): T
    fun visit(node: Negation): T
    fun visit(node: TermChain): T
    fun visit(node: UnresolvedFunctionCall): T
    fun visit(node: UnresolvedReference): T
    fun visit(node: TypeExpression): T
    fun visit(node: FunctionDecl): T
    fun visit(node: FunctionDecl.Parameter): T
    fun visit(node: InitializedVar): T
    fun visit(node: VariableDecl): T
    fun visit(node: FileDecl): T
    fun visit(node: EnumDecl): T
    fun visit(node: EnumDecl.Entry): T
    fun visit(node: ModuleDecl): T
    fun visit(node: StructDecl): T
    fun visit(node: StructDecl.Parameter): T
    fun visit(node: TraitDecl): T
    fun visit(node: IfExpr.ElifExpr): T
    fun visit(node: IfExpr.ElseExpr): T
}

interface NodeVisitor : NodeProcessor<Unit> {
    override fun visit(node: WhileExpr) {}

    override fun visit(node: IfExpr) {}

    override fun visit(node: ExpressionList) {}

    override fun visit(node: CharLiteral) {}

    override fun visit(node: BinaryLiteral) {}

    override fun visit(node: DecimalLiteral) {}

    override fun visit(node: FloatingLiteral) {}

    override fun visit(node: HexLiteral) {}

    override fun visit(node: OctalLiteral) {}

    override fun visit(node: StringLiteral) {}

    override fun visit(node: Assignment) {}

    override fun visit(node: Binary) {}

    override fun visit(node: Comparison) {}

    override fun visit(node: Elvis) {}

    override fun visit(node: Logical) {}

    override fun visit(node: Mathematical) {}

    override fun visit(node: Absolution) {}

    override fun visit(node: BitInversion) {}

    override fun visit(node: Inversion) {}

    override fun visit(node: Negation) {}

    override fun visit(node: TermChain) {}

    override fun visit(node: UnresolvedFunctionCall) {}

    override fun visit(node: UnresolvedReference) {}

    override fun visit(node: TypeExpression) {}

    override fun visit(node: FunctionDecl) {}

    override fun visit(node: FunctionDecl.Parameter) {}

    override fun visit(node: InitializedVar) {}

    override fun visit(node: VariableDecl) {}

    override fun visit(node: FileDecl) {}

    override fun visit(node: EnumDecl) {}

    override fun visit(node: EnumDecl.Entry) {}

    override fun visit(node: ModuleDecl) {}

    override fun visit(node: StructDecl) {}

    override fun visit(node: StructDecl.Parameter) {}

    override fun visit(node: TraitDecl) {}

    override fun visit(node: IfExpr.ElifExpr) {}

    override fun visit(node: IfExpr.ElseExpr) {}
}

interface IntermediateNodeProcessor<T> : NodeProcessor<List<T>> {
    private fun Node.forAll() = listOfNotNull(
        (this as? CodeFlowExpr)?.let(::visit),
        (this as? Term)?.let(::visit),
        (this as? Operation)?.let(::visit),
        (this as? Literal)?.let(::visit),
        (this as? Expression)?.let(::visit),
        (this as? ObjectDecl)?.let(::visit),
        (this as? CallableDecl)?.let(::visit),
        (this as? TypedDecl)?.let(::visit),
        (this as? NamedDecl)?.let(::visit),
        (this as? BlockLevelDecl)?.let(::visit),
        (this as? ObjectLevelDecl)?.let(::visit),
        (this as? TopLevelDecl)?.let(::visit),
        (this as? Leaf)?.let(::visit),
        visit(this)
    )

    fun visit(node: Node): T
    fun visit(node: Leaf): T
    fun visit(node: TopLevelDecl): T
    fun visit(node: ObjectLevelDecl): T
    fun visit(node: BlockLevelDecl): T
    fun visit(node: NamedDecl): T
    fun visit(node: TypedDecl): T
    fun visit(node: CallableDecl): T
    fun visit(node: ObjectDecl): T
    fun visit(node: Expression): T
    fun visit(node: Literal): T
    fun visit(node: Operation): T
    fun visit(node: Term): T
    fun visit(node: CodeFlowExpr): T

    override fun visit(node: WhileExpr): List<T> = node.forAll()
    override fun visit(node: IfExpr): List<T> = node.forAll()
    override fun visit(node: ExpressionList): List<T> = node.forAll()
    override fun visit(node: CharLiteral): List<T> = node.forAll()
    override fun visit(node: BinaryLiteral): List<T> = node.forAll()
    override fun visit(node: DecimalLiteral): List<T> = node.forAll()
    override fun visit(node: FloatingLiteral): List<T> = node.forAll()
    override fun visit(node: HexLiteral): List<T> = node.forAll()
    override fun visit(node: OctalLiteral): List<T> = node.forAll()
    override fun visit(node: StringLiteral): List<T> = node.forAll()
    override fun visit(node: Assignment): List<T> = node.forAll()
    override fun visit(node: Binary): List<T> = node.forAll()
    override fun visit(node: Comparison): List<T> = node.forAll()
    override fun visit(node: Elvis): List<T> = node.forAll()
    override fun visit(node: Logical): List<T> = node.forAll()
    override fun visit(node: Mathematical): List<T> = node.forAll()
    override fun visit(node: Absolution): List<T> = node.forAll()
    override fun visit(node: BitInversion): List<T> = node.forAll()
    override fun visit(node: Inversion): List<T> = node.forAll()
    override fun visit(node: Negation): List<T> = node.forAll()
    override fun visit(node: TermChain): List<T> = node.forAll()
    override fun visit(node: UnresolvedFunctionCall): List<T> = node.forAll()
    override fun visit(node: UnresolvedReference): List<T> = node.forAll()
    override fun visit(node: TypeExpression): List<T> = node.forAll()
    override fun visit(node: FunctionDecl): List<T> = node.forAll()
    override fun visit(node: FunctionDecl.Parameter): List<T> = node.forAll()
    override fun visit(node: InitializedVar): List<T> = node.forAll()
    override fun visit(node: VariableDecl): List<T> = node.forAll()
    override fun visit(node: FileDecl): List<T> = node.forAll()
    override fun visit(node: EnumDecl): List<T> = node.forAll()
    override fun visit(node: EnumDecl.Entry): List<T> = node.forAll()
    override fun visit(node: ModuleDecl): List<T> = node.forAll()
    override fun visit(node: StructDecl): List<T> = node.forAll()
    override fun visit(node: StructDecl.Parameter): List<T> = node.forAll()
    override fun visit(node: TraitDecl): List<T> = node.forAll()
    override fun visit(node: IfExpr.ElifExpr): List<T> = node.forAll()
    override fun visit(node: IfExpr.ElseExpr): List<T> = node.forAll()
}

interface IntermediateNodeVisitor : IntermediateNodeProcessor<Unit> {
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
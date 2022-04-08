@file:Suppress("unused")

package ru.tesserakt.kodept.visitor

import arrow.core.NonEmptyList
import ru.tesserakt.kodept.parser.AST.*

interface NodeVisitor<T> {
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

interface UnitNodeVisitor : NodeVisitor<Unit> {
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

interface IntermediateNodeVisitor<T> : NodeVisitor<List<T>> {
    fun visit(node: Node): T
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

    override fun visit(node: IfExpr.ElseExpr) = listOf(visit(node as Node))
    override fun visit(node: IfExpr.ElifExpr) = listOf(visit(node as Node))
    override fun visit(node: TraitDecl) = listOf(
        visit(node as ObjectDecl),
        visit(node as TopLevelDecl),
        visit(node as NamedDecl),
        visit(node as Node),
    )

    override fun visit(node: StructDecl.Parameter) = listOf(
        visit(node as NamedDecl),
        visit(node as Node),
    )

    override fun visit(node: StructDecl) = listOf(
        visit(node as ObjectDecl),
        visit(node as TopLevelDecl),
        visit(node as NamedDecl),
        visit(node as Node),
    )

    override fun visit(node: ModuleDecl) = listOf(
        visit(node as NamedDecl),
        visit(node as Node),
    )

    override fun visit(node: EnumDecl.Entry) = listOf(
        visit(node as ObjectDecl),
        visit(node as NamedDecl),
        visit(node as Node),
    )

    override fun visit(node: EnumDecl) = listOf(
        visit(node as ObjectDecl),
        visit(node as TopLevelDecl),
        visit(node as NamedDecl),
        visit(node as Node),
    )

    override fun visit(node: FileDecl) = listOf(visit(node as Node))
    override fun visit(node: VariableDecl) = listOf(
        visit(node as BlockLevelDecl),
        visit(node as CallableDecl),
        visit(node as NamedDecl),
        visit(node as Node),
    )

    override fun visit(node: InitializedVar) = listOf(
        visit(node as BlockLevelDecl),
        visit(node as CallableDecl),
        visit(node as NamedDecl),
        visit(node as Node),
    )

    override fun visit(node: FunctionDecl.Parameter) = listOf(visit(node as Node))
    override fun visit(node: FunctionDecl) = listOf(
        visit(node as ObjectLevelDecl),
        visit(node as TopLevelDecl),
        visit(node as BlockLevelDecl),
        visit(node as CallableDecl),
        visit(node as NamedDecl),
        visit(node as Node),
    )

    override fun visit(node: TypeExpression) = listOf(
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as TypedDecl),
        visit(node as Node),
    )

    override fun visit(node: UnresolvedReference) = listOf(
        visit(node as Term),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: UnresolvedFunctionCall) = listOf(
        visit(node as Term),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: TermChain) = listOf(
        visit(node as Term),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: Negation) = listOf(
        visit(node as Operation),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: Inversion) = listOf(
        visit(node as Operation),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: BitInversion) = listOf(
        visit(node as Operation),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: Absolution) = listOf(
        visit(node as Operation),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: Mathematical) = listOf(
        visit(node as Operation),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: Logical) = listOf(
        visit(node as Operation),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: Elvis) = listOf(
        visit(node as Operation),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: Comparison) = listOf(
        visit(node as Operation),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: Binary) = listOf(
        visit(node as Operation),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: Assignment) = listOf(
        visit(node as Operation),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: StringLiteral) = listOf(
        visit(node as Literal),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: OctalLiteral) = listOf(
        visit(node as Literal),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: HexLiteral) = listOf(
        visit(node as Literal),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: FloatingLiteral) = listOf(
        visit(node as Literal),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: DecimalLiteral) = listOf(
        visit(node as Literal),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: BinaryLiteral) = listOf(
        visit(node as Literal),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: CharLiteral) = listOf(
        visit(node as Literal),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: ExpressionList) = listOf(
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: IfExpr) = listOf(
        visit(node as CodeFlowExpr),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )

    override fun visit(node: WhileExpr) = listOf(
        visit(node as CodeFlowExpr),
        visit(node as Expression),
        visit(node as BlockLevelDecl),
        visit(node as Node),
    )
}

interface UnitIntermediateNodeVisitor : IntermediateNodeVisitor<Unit> {
    override fun visit(node: Node) {}
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

interface Acceptable {
    fun <T> accept(visitor: NodeVisitor<T>): T

    fun <T> acceptRecursively(visitor: NodeVisitor<T>): NonEmptyList<T>
}
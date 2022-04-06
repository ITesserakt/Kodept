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

interface IntermediateNodeVisitor : UnitNodeVisitor {
    fun visit(node: Node) {}
    fun visit(node: TopLevelDecl) {}
    fun visit(node: ObjectLevelDecl) {}
    fun visit(node: BlockLevelDecl) {}
    fun visit(node: NamedDecl) {}
    fun visit(node: TypedDecl) {}
    fun visit(node: CallableDecl) {}
    fun visit(node: ObjectDecl) {}
    fun visit(node: Expression) {}
    fun visit(node: Literal) {}
    fun visit(node: Operation) {}
    fun visit(node: Term) {}
    fun visit(node: CodeFlowExpr) {}

    override fun visit(node: IfExpr.ElseExpr) = visit(node as Node)
    override fun visit(node: IfExpr.ElifExpr) = visit(node as Node)
    override fun visit(node: TraitDecl) {
        visit(node as ObjectDecl)
        visit(node as TopLevelDecl)
        visit(node as NamedDecl)
        visit(node as Node)
    }

    override fun visit(node: StructDecl.Parameter) {
        visit(node as NamedDecl)
        visit(node as Node)
    }

    override fun visit(node: StructDecl) {
        visit(node as ObjectDecl)
        visit(node as TopLevelDecl)
        visit(node as NamedDecl)
        visit(node as Node)
    }

    override fun visit(node: ModuleDecl) {
        visit(node as NamedDecl)
        visit(node as Node)
    }

    override fun visit(node: EnumDecl.Entry) {
        visit(node as ObjectDecl)
        visit(node as NamedDecl)
        visit(node as Node)
    }

    override fun visit(node: EnumDecl) {
        visit(node as ObjectDecl)
        visit(node as TopLevelDecl)
        visit(node as NamedDecl)
        visit(node as Node)
    }

    override fun visit(node: FileDecl) = visit(node as Node)
    override fun visit(node: VariableDecl) {
        visit(node as BlockLevelDecl)
        visit(node as CallableDecl)
        visit(node as NamedDecl)
        visit(node as Node)
    }

    override fun visit(node: InitializedVar) {
        visit(node as BlockLevelDecl)
        visit(node as CallableDecl)
        visit(node as NamedDecl)
        visit(node as Node)
    }

    override fun visit(node: FunctionDecl.Parameter) = visit(node as Node)
    override fun visit(node: FunctionDecl) {
        visit(node as ObjectLevelDecl)
        visit(node as TopLevelDecl)
        visit(node as BlockLevelDecl)
        visit(node as CallableDecl)
        visit(node as NamedDecl)
        visit(node as Node)
    }

    override fun visit(node: TypeExpression) {
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as TypedDecl)
        visit(node as Node)
    }

    override fun visit(node: UnresolvedReference) {
        visit(node as Term)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: UnresolvedFunctionCall) {
        visit(node as Term)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: TermChain) {
        visit(node as Term)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: Negation) {
        visit(node as Operation)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: Inversion) {
        visit(node as Operation)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: BitInversion) {
        visit(node as Operation)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: Absolution) {
        visit(node as Operation)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: Mathematical) {
        visit(node as Operation)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: Logical) {
        visit(node as Operation)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: Elvis) {
        visit(node as Operation)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: Comparison) {
        visit(node as Operation)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: Binary) {
        visit(node as Operation)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: Assignment) {
        visit(node as Operation)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: StringLiteral) {
        visit(node as Literal)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: OctalLiteral) {
        visit(node as Literal)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: HexLiteral) {
        visit(node as Literal)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: FloatingLiteral) {
        visit(node as Literal)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: DecimalLiteral) {
        visit(node as Literal)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: BinaryLiteral) {
        visit(node as Literal)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: CharLiteral) {
        visit(node as Literal)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: ExpressionList) {
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: IfExpr) {
        visit(node as CodeFlowExpr)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }

    override fun visit(node: WhileExpr) {
        visit(node as CodeFlowExpr)
        visit(node as Expression)
        visit(node as BlockLevelDecl)
        visit(node as Node)
    }
}

interface Acceptable {
    fun <T> accept(visitor: NodeVisitor<T>): T

    fun <T> acceptRecursively(visitor: NodeVisitor<T>): NonEmptyList<T>
}
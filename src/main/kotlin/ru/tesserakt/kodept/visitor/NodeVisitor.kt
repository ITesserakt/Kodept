package ru.tesserakt.kodept.visitor

import ru.tesserakt.kodept.parser.AST.*

interface NodeVisitor {
    fun visit(node: WhileExpr) {}
    fun visit(node: IfExpr) {}
    fun visit(node: ExpressionList) {}
    fun visit(node: CharLiteral) {}
    fun visit(node: BinaryLiteral) {}
    fun visit(node: DecimalLiteral) {}
    fun visit(node: FloatingLiteral) {}
    fun visit(node: HexLiteral) {}
    fun visit(node: OctalLiteral) {}
    fun visit(node: StringLiteral) {}
    fun visit(node: Assignment) {}
    fun visit(node: Binary) {}
    fun visit(node: Comparison) {}
    fun visit(node: Elvis) {}
    fun visit(node: Logical) {}
    fun visit(node: Mathematical) {}
    fun visit(node: Absolution) {}
    fun visit(node: BitInversion) {}
    fun visit(node: Inversion) {}
    fun visit(node: Negation) {}
    fun visit(node: TermChain) {}
    fun visit(node: UnresolvedFunctionCall) {}
    fun visit(node: UnresolvedReference) {}
    fun visit(node: TypeExpression) {}
    fun visit(node: FunctionDecl) {}
    fun visit(node: FunctionDecl.Parameter) {}
    fun visit(node: InitializedVar) {}
    fun visit(node: VariableDecl) {}
    fun visit(node: FileDecl) {}
    fun visit(node: EnumDecl) {}
    fun visit(node: EnumDecl.Entry) {}
    fun visit(node: ModuleDecl) {}
    fun visit(node: StructDecl) {}
    fun visit(node: StructDecl.Parameter) {}
    fun visit(node: TraitDecl) {}
    fun visit(node: IfExpr.ElifExpr) {}
    fun visit(node: IfExpr.ElseExpr) {}
}

interface IntermediateNodeVisitor : NodeVisitor {
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
    fun accept(visitor: NodeVisitor)
}
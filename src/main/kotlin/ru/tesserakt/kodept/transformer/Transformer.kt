package ru.tesserakt.kodept.transformer

import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.visitor.NodeProcessor

abstract class Transformer : NodeProcessor<AST.Node>() {
    override fun default(node: AST.Node): AST.Node = node
}

@Suppress("unchecked_cast")
fun <N : AST.Node> N.acceptTransform(transformer: Transformer): N = when (this) {
    is AST.IfExpr -> transformer.visit(this)
    is AST.WhileExpr -> transformer.visit(this)
    is AST.ExpressionList -> transformer.visit(this)
    is AST.CharLiteral -> transformer.visit(this)
    is AST.BinaryLiteral -> transformer.visit(this)
    is AST.DecimalLiteral -> transformer.visit(this)
    is AST.FloatingLiteral -> transformer.visit(this)
    is AST.HexLiteral -> transformer.visit(this)
    is AST.OctalLiteral -> transformer.visit(this)
    is AST.StringLiteral -> transformer.visit(this)
    is AST.Assignment -> transformer.visit(this)
    is AST.Binary -> transformer.visit(this)
    is AST.Comparison -> transformer.visit(this)
    is AST.Elvis -> transformer.visit(this)
    is AST.Logical -> transformer.visit(this)
    is AST.Mathematical -> transformer.visit(this)
    is AST.Absolution -> transformer.visit(this)
    is AST.BitInversion -> transformer.visit(this)
    is AST.Inversion -> transformer.visit(this)
    is AST.Negation -> transformer.visit(this)
    is AST.TermChain -> transformer.visit(this)
    is AST.FunctionCall -> transformer.visit(this)
    is AST.Reference -> transformer.visit(this)
    is AST.TypeExpression -> transformer.visit(this)
    is AST.FunctionDecl -> transformer.visit(this)
    is AST.InitializedVar -> transformer.visit(this)
    is AST.VariableDecl -> transformer.visit(this)
    is AST.IfExpr.ElifExpr -> transformer.visit(this)
    is AST.IfExpr.ElseExpr -> transformer.visit(this)
    is AST.FileDecl -> transformer.visit(this)
    is AST.EnumDecl.Entry -> transformer.visit(this)
    is AST.Parameter -> transformer.visit(this)
    is AST.EnumDecl -> transformer.visit(this)
    is AST.ModuleDecl -> transformer.visit(this)
    is AST.StructDecl -> transformer.visit(this)
    is AST.TraitDecl -> transformer.visit(this)
    is AST.TypeReference -> transformer.visit(this)
    is AST.TupleLiteral -> transformer.visit(this)
    is AST.InferredParameter -> transformer.visit(this)
    is AST.AbstractFunctionDecl -> transformer.visit(this)
    else -> throw IllegalStateException("Impossible")
} as N
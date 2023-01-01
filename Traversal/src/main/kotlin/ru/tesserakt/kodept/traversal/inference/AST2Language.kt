package ru.tesserakt.kodept.traversal.inference

import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.traversal.BinaryOperatorDesugaring
import ru.tesserakt.kodept.traversal.ReferenceResolver
import ru.tesserakt.kodept.traversal.TypeReferenceResolver
import ru.tesserakt.kodept.traversal.UnaryOperatorDesugaring

fun AST.Expression.convert(): Language = when (this) {
    is AST.BinaryOperator -> BinaryOperatorDesugaring.contract(this)
    is AST.UnaryOperator -> UnaryOperatorDesugaring.contract(this)
    is AST.ExpressionList -> {
        val list = mutableListOf<Language>()
        var i = 0
        while (i < expressions.size) {
            when (val item = expressions[i]) {
                is AST.InitializedVar -> {
                    val next = expressions.getOrNull(i + 1)
                    val usage = if (next is AST.Expression) {
                        i++
                        next.convert()
                    } else {
                        Language.Literal.unit
                    }
                    list += Language.Let(
                        item.expr.convert(),
                        Language.Var((item.reference as AST.ResolvedReference).mangle()),
                        usage
                    )
                }
                is AST.Expression -> {
                    list += item.convert()
                }
                is AST.Assignment -> {
                    list += item.left.convert()
                    list += item.right.convert()
                }
                is AST.WhileExpr -> {
                    list += item.condition.convert()
                    list += item.body.convert()
                }
                is AST.FunctionDecl -> {}
            }
            i++
        }
        if (expressions.size == 1) (expressions.first() as AST.Expression).convert()
        else list.lastOrNull() ?: Language.Literal.unit
    }
    is AST.IfExpr -> TODO()
    is AST.LambdaExpr -> Language.Lambda.uncurry(params.map { it.convert() }, body.convert())
    is AST.BinaryLiteral -> Language.Literal.Number(value)
    is AST.CharLiteral -> TODO()
    is AST.DecimalLiteral -> Language.Literal.Number(value)
    is AST.FloatingLiteral -> Language.Literal.Floating(value)
    is AST.HexLiteral -> Language.Literal.Number(value)
    is AST.OctalLiteral -> Language.Literal.Number(value)
    is AST.StringLiteral -> TODO()
    is AST.TupleLiteral -> Language.Literal.Tuple(items.map { it.convert() })
    is AST.FunctionCall -> Language.App.curry(params.map { it.convert() }, reference.convert())
    is AST.ResolvedReference -> Language.Var(mangle())
    is AST.ResolvedTypeReference -> TODO()
    is AST.Reference -> ReferenceResolver.contract(this)
    is AST.TypeReference -> TypeReferenceResolver.contract(this)
}

fun AST.FunctionDecl.convert(): Pair<Language, Assumptions> {
    val params = params.map { Language.Var(it.name) }
    val ctxWithParams = Assumptions(params.associateWith { MonomorphicType.Var() })
    return Language.Lambda.uncurry(params, rest.convert()) to ctxWithParams
}

fun AST.InferredParameter.convert() = Language.Var(mangle())

fun AST.InferredParameter.mangle() = name

fun AST.ResolvedReference.mangle() = name


package ru.tesserakt.kodept.traversal.inference

import arrow.core.*
import arrow.typeclasses.Semigroup
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.error.CompilerCrash
import ru.tesserakt.kodept.traversal.BinaryOperatorDesugaring
import ru.tesserakt.kodept.traversal.ReferenceResolver
import ru.tesserakt.kodept.traversal.TypeReferenceResolver
import ru.tesserakt.kodept.traversal.UnaryOperatorDesugaring
import ru.tesserakt.kodept.traversal.inference.Assumptions.Companion.fold
import kotlin.math.exp

fun AST.Expression.convert(): Pair<Assumptions, Language> = when (this) {
    is AST.BinaryOperator -> BinaryOperatorDesugaring.contract(this)
    is AST.UnaryOperator -> UnaryOperatorDesugaring.contract(this)
    is AST.ExpressionList -> {
        var ctx = Assumptions.empty()
        val list = mutableListOf<Language>()
        var i = 0
        while (i < expressions.size) {
            when (val item = expressions[i]) {
                is AST.InitializedVar -> {
                    val next = expressions.getOrNull(i + 1)
                    val (usageCtx, usage) = if (next is AST.Expression) {
                        i++
                        next.convert()
                    } else {
                        Assumptions.empty() to Language.Literal.unit
                    }
                    item.expr.convert().let {
                        list += Language.Let(
                            it.second, Language.Var((item.reference as AST.ResolvedReference).mangle()), usage
                        )
                        ctx = ctx.combine(it.first).combine(usageCtx)
                    }
                }
                is AST.Expression -> {
                    item.convert().let {
                        list += it.second
                        ctx = ctx.combine(it.first)
                    }
                }
                is AST.Assignment -> {
                    val (c1, l1) = item.left.convert()
                    val (c2, l2) = item.right.convert()
                    val (s1, t1) = (l1 infer c1.combine(c2)).fold({ throw CompilerCrash(it.toString()) }, ::identity)
                    val (s2, t2) = (l2 infer c1.combine(c2).combine(s1)).fold(
                        { throw CompilerCrash(it.toString()) }, ::identity
                    )
                    val s3 = (t1 unify t2).fold({ throw CompilerCrash(it.toString()) }, ::identity)
                    list += listOf(l1, l2)
                    ctx = ctx.combine(c1).combine(c2).combine(s1).combine(s2).substitute(s3)
                }
                is AST.WhileExpr -> {
                    val (c1, l1) = item.condition.convert()
                    val (c2, l2) = item.body.convert()
                    val (s1, t1) = (l1 infer c1.combine(c2)).fold({ throw CompilerCrash(it.toString()) }, ::identity)
                    val s2 = (t1 unify PrimitiveType.Bool).fold({ throw CompilerCrash(it.toString()) }, ::identity)
                    list += listOf(l1, l2)
                    ctx = ctx.combine(c1).combine(c2).combine(s1).substitute(s2)
                }
                is AST.FunctionDecl -> {}
            }
            i++
        }
        if (expressions.size == 1) (expressions.first() as AST.Expression).convert()
        else ctx to (list.reduceRightIndexedOrNull { idx, acc, next ->
            Language.Let(
                acc, Language.Var(idx.toString()), next
            )
        } ?: Language.Literal.unit)
    }
    is AST.IfExpr -> {
        val (c1, l1) = condition.convert()
        val (c2, l2) = body.convert()
        val (co, bo) = elifs.unzip { it.condition.convert() to it.body.convert() }
        val (cco, llo) = co.unzip().mapLeft { it.fold() }
        val (cbo, lbo) = bo.unzip().mapLeft { it.fold() }
        val (c3, l3) = el?.body?.convert() ?: Pair(Assumptions.empty(), null)
        c3.combine(cbo).combine(cco).combine(c2).combine(c1) to Language.If(l1,
            l2,
            llo.zip(lbo).fold(l3 ?: Language.Literal.unit) { acc, (cond, body) ->
                Language.If(cond, body, acc)
            })
    }
    is AST.LambdaExpr -> {
        val (ctx, body) = body.convert()
        val (paramCtx, params) = params.map { it.convert() }.unzip()
        paramCtx.fold().combine(ctx) to Language.Lambda.uncurry(params, body)
    }
    is AST.BinaryLiteral -> Assumptions.empty() to Language.Literal.Number(value)
    is AST.CharLiteral -> TODO()
    is AST.DecimalLiteral -> Assumptions.empty() to Language.Literal.Number(value)
    is AST.FloatingLiteral -> Assumptions.empty() to Language.Literal.Floating(value)
    is AST.HexLiteral -> Assumptions.empty() to Language.Literal.Number(value)
    is AST.OctalLiteral -> Assumptions.empty() to Language.Literal.Number(value)
    is AST.StringLiteral -> TODO()
    is AST.TupleLiteral -> {
        val (itemsCtx, items) = items.map { it.convert() }.unzip()
        itemsCtx.fold() to Language.Literal.Tuple(items)
    }
    is AST.FunctionCall -> {
        val (paramCtx, params) = params.map { it.convert() }.unzip()
        val (refCtx, reference) = reference.convert()
        paramCtx.fold().combine(refCtx) to Language.App.curry(params, reference)
    }
    is AST.ResolvedReference -> when (val r = referral) {
        is AST.InferredParameter -> r.convert().first to Language.Var(mangle())
        is AST.InitializedVar -> {
            val v = Language.Var(mangle())
            if (r.type == null) Assumptions.empty() to v
            else Assumptions(v to r.type!!.evalType()) to v
        }
        is AST.ForeignFunctionDecl -> {
            val v = Language.Var(mangle())
            val (ctx, params) = r.params.map { it.convert() }.unzip().mapLeft { it.fold() }
            val nonEmptyParams = NonEmptyList.fromList(params).getOrElse { nonEmptyListOf(Language.Literal.unit) }
            val types = NonEmptyList.fromListUnsafe(List(params.size) { MonomorphicType.Var() })
            val ret = r.returns?.evalType() ?: MonomorphicType.Tuple.unit
            Assumptions(v to MonomorphicType.Fn.uncurry(types, ret), *nonEmptyParams.zip(types).toTypedArray()).combine(
                ctx
            ) to v
        }
        else -> Assumptions.empty() to Language.Var(mangle())
    }
    is AST.ResolvedTypeReference -> {
        val r = Language.Var(mangle())
        Assumptions(r to constantTypes.getOrPut(referral.name) { MonomorphicType.Constant(referral.name) }) to r
    }
    is AST.Reference -> ReferenceResolver.contract(this)
    is AST.TypeReference -> TypeReferenceResolver.contract(this)
    is AST.Intrinsics.AccessVariable -> Assumptions.empty() to Language.TypedMagic(this, variable.type.evalType())
    is AST.Intrinsics.Construct -> Assumptions.empty() to Language.TypedMagic(this,
        constantTypes.getOrPut(obj.name) { MonomorphicType.Constant(obj.name) })
}

internal fun AST.FunctionDecl.convert(): Pair<Language, Assumptions> {
    val (ctxWithParams, params) = params.map { it.convert() }.unzip()
    val (ctx, body) = rest.convert()
    return Language.Lambda.uncurry(params, body) to ctxWithParams.fold().combine(ctx)
}

private fun AST.InferredParameter.convert() = if (type == null) Assumptions.empty() to Language.Var(mangle())
else {
    val p = Language.Var(mangle())
    Assumptions(p to type!!.evalType()) to p
}

fun AST.InferredParameter.mangle() = "$name\$$id"
fun AST.ResolvedTypeReference.mangle() = "$name\$${referral.id}"
fun AST.ResolvedReference.mangle() = "$name\$${referral.id}"

private val constantTypes: MutableMap<String, MonomorphicType> = mutableMapOf()

private fun AST.TypeLike.evalType(): MonomorphicType = when (this) {
    is AST.TupleType -> MonomorphicType.Tuple(items.map { it.evalType() })
    is AST.ResolvedTypeReference -> when (val r = referral) {
        is AST.EnumDecl -> constantTypes.getOrPut(r.name) {
            MonomorphicType.Union(r.enumEntries.map {
                constantTypes.getOrPut(it.name) { MonomorphicType.Constant(it.name) }
            })
        }
        else -> constantTypes.getOrPut(r.name) { MonomorphicType.Constant(r.name) }
    }
    is AST.TypeReference -> TypeReferenceResolver.contract()
    is AST.UnionType -> TODO()
}
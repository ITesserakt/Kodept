package ru.tesserakt.kodept.traversal.inference

import arrow.core.flatten
import arrow.core.map
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.traversal.*
import ru.tesserakt.kodept.traversal.inference.TypeInferenceAnalyzer.type

typealias TypeContext = Map<String, TypeScheme>
typealias Substitutions = Set<Substitution>

data class UnboundReference(val ref: AST.WithResolutionContext) : Exception()

object TypeInference {
    private fun Type.withoutSubstitutions() = emptySet<Substitution>() to this
    private fun TypeContext.applySubstitutions(substitutions: Substitutions): TypeContext =
        mapValues { it.value.applySubstitutions(substitutions) }

    context (ReportCollector, Filepath)
            private fun List<AST.Expression>.inferFold(context: TypeContext, substitutions: Substitutions) =
        fold(substitutions to emptyList<Type>()) { (s1, tArgs), next ->
            val (s2, tArg) = next.infer(context.applySubstitutions(s1))
            s1 compose s2 to tArgs + tArg
        }

    context (ReportCollector, Filepath)
            private fun AST.Literal.inferLiteral(context: TypeContext) = when (this) {
        is AST.BinaryLiteral -> Type.number.withoutSubstitutions()
        is AST.CharLiteral -> Type.char.withoutSubstitutions()
        is AST.DecimalLiteral -> Type.number.withoutSubstitutions()
        is AST.FloatingLiteral -> Type.floating.withoutSubstitutions()
        is AST.HexLiteral -> Type.number.withoutSubstitutions()
        is AST.OctalLiteral -> Type.number.withoutSubstitutions()
        is AST.StringLiteral -> Type.string.withoutSubstitutions()
        is AST.TupleLiteral -> items.inferFold(context, emptySet()).map(Type::Tuple)
    }

    context (ReportCollector, Filepath)
    fun AST.Expression.infer(context: TypeContext): Pair<Substitutions, Type> = when (this) {
        is AST.Dereference -> DereferenceEliminator.contract()
        is AST.BinaryOperator -> BinaryOperatorDesugaring.contract()
        is AST.UnaryOperator -> UnaryOperatorDesugaring.contract()

        is AST.Literal -> inferLiteral(context)

        is AST.ExpressionList ->
            expressions.fold(Triple(context, emptySet<Substitution>(), null as? Type)) { (ctx, s0, _), next ->
                when (next) {
                    is AST.InitializedVar -> {
                        val (s1, tRef) = next.expr.infer(context)
                        val ref = next.reference as AST.ResolvedReference
                        val newCtx = ctx + (ref.toString() to TypeScheme(emptySet(), tRef.applySubstitutions(s1)))
                        Triple(newCtx, s1 compose s0, null)
                    }
                    is AST.WhileExpr -> {
                        val (s1, tCondition) = next.condition.infer(ctx.applySubstitutions(s0))
                        val s2 = TypeEquation(next.condition, tCondition, Type.bool).unify()
                        val (s3, _) = next.body.infer(ctx.applySubstitutions(s2 compose s1 compose s0))
                        Triple(ctx, s3 compose s2 compose s1 compose s0, null)
                    }
                    is AST.Statement -> Triple(ctx, s0, null)
                    is AST.Expression -> {
                        val (s1, tBody) = next.infer(ctx.applySubstitutions(s0))
                        Triple(ctx, s1 compose s0, tBody)
                    }
                }
            }.let { it.second to (it.third ?: Type.unit) }
        is AST.IfExpr -> {
            val (s0, tCondition) = condition.infer(context)
            val s1 = TypeEquation(this, tCondition, Type.bool).unify()
            val (s2, tBody) = body.infer(context.applySubstitutions(s1 compose s0))

            val (s3, tElifs) = elifs.fold(s2 compose s1 to emptyList<Type>()) { (se1, acc), next ->
                val (se2, tConditionE) = next.condition.infer(context.applySubstitutions(se1))
                val se3 = TypeEquation(next.condition, tConditionE, Type.bool).unify()
                next.body.infer(context.applySubstitutions(se3 compose se2)).map { acc + it }
            }.map { it.zip(elifs.map(AST.IfExpr.ElifExpr::body)) }

            val (s4, tElse) = el?.body?.infer(context.applySubstitutions(s3 compose s2 compose s1))
                ?: Type.unit.withoutSubstitutions()

            val s5 = s4 compose s3
            val branches = (listOf(tBody to body) + tElifs + (tElse to (el?.body
                ?: this))).map { it.first.applySubstitutions(s5) to it.second }
            val s6 = branches.zipWithNext { (a, n1), (b, n2) ->
                listOf(TypeEquation(n1, a, b), TypeEquation(n2, b, a))
            }.flatten().fold(emptySet<Substitution>()) { acc, next ->
                acc + next.copy(a = next.a.applySubstitutions(acc), b = next.b.applySubstitutions(acc)).unify()
            }
            s6 compose s5 to tBody.applySubstitutions(s6)
        }
        is AST.LambdaExpr -> {
            val tParams = params.associate { it.name to TypeScheme(emptySet(), it.type?.type() ?: Type.T()) }
            val newCtx = context + tParams
            val (s1, tBody) = body.infer(newCtx)
            val s2 = when (val ret = returns) {
                null -> emptySet()
                else -> TypeEquation(body, tBody, ret.type()).unify()
            }
            s2 compose s1 to Type.Fn.fromParams(tParams.map { it.value.type.applySubstitutions(s1) }, tBody)
        }
        is AST.FunctionCall -> {
            val tRes = Type.T()
            val (s1, tFun) = reference.infer(context)
            val (s2, tArgs) = params.inferFold(context.applySubstitutions(s1), s1)
            val s3 = TypeEquation(this, tFun.applySubstitutions(s2 compose s1), Type.Fn.fromParams(tArgs, tRes)).unify()

            s3 compose s2 compose s1 to tRes.applySubstitutions(s3)
        }
        is AST.ResolvedReference -> (context[fullPath]?.instantiate() ?: throw UnboundReference(this))
            .withoutSubstitutions()
        is AST.ResolvedTypeReference -> (context[fullPath]?.instantiate() ?: throw UnboundReference(this))
            .withoutSubstitutions()

        is AST.Reference -> ReferenceResolver.contract()
        is AST.TypeReference -> TypeReferenceResolver.contract()
    }
}
package ru.tesserakt.kodept.traversal.inference

import arrow.core.continuations.eagerEffect
import arrow.core.identity
import arrow.core.nel
import arrow.core.nonEmptyListOf
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Tree
import ru.tesserakt.kodept.core.walkDownTop
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.traversal.*
import ru.tesserakt.kodept.traversal.inference.Type.*
import ru.tesserakt.kodept.traversal.inference.Type.Companion.bool
import ru.tesserakt.kodept.traversal.inference.Type.Companion.char
import ru.tesserakt.kodept.traversal.inference.Type.Companion.floating
import ru.tesserakt.kodept.traversal.inference.Type.Companion.number
import ru.tesserakt.kodept.traversal.inference.Type.Companion.string
import ru.tesserakt.kodept.traversal.inference.Type.Companion.tag
import ru.tesserakt.kodept.traversal.inference.Type.Companion.unit
import ru.tesserakt.kodept.traversal.inference.Type.Enum
import kotlin.properties.Delegates

object TypeInferenceAnalyzer : Analyzer() {
    init {
        dependsOn(
            BinaryOperatorDesugaring,
            ReferenceResolver,
            TypeReferenceResolver,
            UnaryOperatorDesugaring,
            DereferenceEliminator,
            Function2LambdaTransformer
        )
    }

    private data class ExpressionWithStrictParent(val value: AST.Expression) {
        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (javaClass != other?.javaClass) return false

            other as ExpressionWithStrictParent

            if (value != other.value) return false
            if (value.parent != other.value.parent) return false

            return true
        }

        override fun hashCode(): Int {
            return 31 * value.hashCode() + value.parent.hashCode()
        }

        operator fun invoke() = value
    }

    private var types: MutableMap<ExpressionWithStrictParent, Type> = hashMapOf()
    private val enumTypes = mutableMapOf<AST.EnumDecl, Type>()

    private operator fun AST.Expression.invoke() = ExpressionWithStrictParent(this)
    private val AST.Expression.type get() = types[this()]!!

    private fun AST.TypeReferable.type(): Type = when (this) {
        is AST.EnumDecl.Entry -> tag(name, lazy { enumTypes[parent as AST.EnumDecl]!! })
        is AST.EnumDecl -> Enum(name, enumEntries.map { it.type() }).also { enumTypes[this] = it }
        is AST.ForeignStructDecl -> tag(name)
        is AST.StructDecl -> {
            var params by Delegates.notNull<List<Type>>()
            val structType = Struct(name, lazy { params })
            params = alloc.map {
                if (it.type is AST.ResolvedTypeReference && (it.type as AST.ResolvedTypeReference).referral == this)
                    structType
                else it.type.type()
            }
            structType
        }
        is AST.TraitDecl -> Interface(name)
    }

    fun AST.TypeLike.type(): Type = when (this) {
        is AST.TupleType -> Tuple(items.map { it.type() })
        is AST.ResolvedTypeReference -> referral.type()
        is AST.UnionType -> Union(items.map { it.type() })

        is AST.TypeReference -> TypeReferenceResolver.contract()
    }

    private fun AST.Referable.type(): Type? = when (this) {
        is AST.AbstractFunctionDecl -> Fn.fromParams(params.map { it.type()!! }, returns?.type() ?: unit)
        is AST.ForeignFunctionDecl -> Fn.fromParams(params.map { it.type()!! }, returns?.type() ?: unit)
        is AST.FunctionDecl ->
            if (params.any { it.type == null } && returns != null) Fn(T(), returns!!.type())
            else if (returns != null) Fn.fromParams(params.map { it.type()!! }, returns!!.type())
            else null
        is AST.InferredParameter -> type?.type()
        is AST.InitializedVar -> type?.type()
    }

    private fun generateEquations(aexpr: AnnotatedExpression) = with(aexpr.expr) {
        when (this) {
            is AST.BinaryOperator -> BinaryOperatorDesugaring.contract()
            is AST.UnaryOperator -> UnaryOperatorDesugaring.contract()
            is AST.Dereference -> DereferenceEliminator.contract()
            is AST.ExpressionList -> when (val last = expressions.last()) {
                is AST.Statement -> TypeEquation(aexpr, unit)
                is AST.Expression -> TypeEquation(aexpr, last.type)
            }.nel()
            is AST.IfExpr -> nonEmptyListOf(TypeEquation(condition, condition.type, bool),
                el?.body?.let { TypeEquation(aexpr, it.type) } ?: TypeEquation(aexpr, Bottom),
                TypeEquation(aexpr, body.type)
            ) + elifs.flatMap {
                nonEmptyListOf(
                    TypeEquation(it.condition, it.condition.type, bool), TypeEquation(aexpr, it.body.type)
                )
            }
            is AST.BinaryLiteral -> TypeEquation(aexpr, number).nel()
            is AST.CharLiteral -> TypeEquation(aexpr, char).nel()
            is AST.DecimalLiteral -> TypeEquation(aexpr, number).nel()
            is AST.FloatingLiteral -> TypeEquation(aexpr, floating).nel()
            is AST.HexLiteral -> TypeEquation(aexpr, number).nel()
            is AST.OctalLiteral -> TypeEquation(aexpr, number).nel()
            is AST.StringLiteral -> TypeEquation(aexpr, string).nel()
            is AST.TupleLiteral -> TypeEquation(aexpr, Tuple(items.map { it.type })).nel()
            is AST.FunctionCall -> TypeEquation(
                reference,
                reference.type,
                Fn.fromParams(params.map { it.type }, type)
            ).nel()
            is AST.ResolvedReference -> referral.type()?.let { TypeEquation(aexpr, it).nel() } ?: emptyList()
            is AST.ResolvedTypeReference -> TypeEquation(aexpr, referral.type()).nel()
            is AST.LambdaExpr -> TypeEquation(aexpr, Fn.fromParams(params.map { it.type() ?: T() }, body.type)).nel()

            is AST.Reference -> ReferenceResolver.contract()
            is AST.TypeReference -> TypeReferenceResolver.contract()
        }
    }

    private fun populateContext(ast: AST) = ast.walkThrough {
        when (it) {
            is AST.ResolvedReference -> if (it.referral is AST.FunctionLike)
                it.fullPath to TypeScheme(emptySet(), it.referral.type() ?: T())
            else null
            is AST.ResolvedTypeReference -> it.fullPath to TypeScheme(emptySet(), it.referral.type())
            else -> null
        }
    }.filterNotNull().toMap()

    override fun ReportCollector.analyze(ast: AST) = eagerEffect {
        val expressions = ast.fastFlatten(Tree.SearchMode.LevelOrder).filterIsInstance<AST.LambdaExpr>()
        var context = populateContext(ast).toMutableMap()

        expressions.filter {
            it.walkDownTop(::identity).drop(1).filterIsInstance<AST.LambdaExpr>().firstOrNull() == null
        }.forEach {
            with(ast.filepath) {
                val type = TypeInference.runCatching {
                    val (subst, type) = it.infer(context)
                    context = context.mapValues { it.value.applySubstitutions(subst) }.toMutableMap()
                    type
                }.bind {
                    if (it is UnboundReference) UnrecoverableError(
                        it.ref.rlt.position.nel(),
                        Report.Severity.ERROR,
                        SemanticError.ReferenceCannotBeTyped(it.ref.toString())
                    ) else throw IllegalStateException(it)
                }
                if (hasErrors)
                    failWithReport(it.rlt.position.nel(), Report.Severity.ERROR, SemanticError.TypeInferenceFailed)

                if (it.parent is AST.FunctionLike) {
                    val fn = it.parent as AST.FunctionLike
                    println("Function `${fn.name}` has type $type")
                } else {
                    println("Lambda at ${it.rlt.position} has type $type")
                }
            }
        }
    }
}
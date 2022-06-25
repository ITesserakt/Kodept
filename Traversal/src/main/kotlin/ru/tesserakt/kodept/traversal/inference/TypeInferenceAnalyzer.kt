package ru.tesserakt.kodept.traversal.inference

import arrow.core.continuations.eagerEffect
import arrow.core.identity
import arrow.core.nel
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Tree
import ru.tesserakt.kodept.core.walkDownTop
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.error.SemanticNote
import ru.tesserakt.kodept.traversal.*
import ru.tesserakt.kodept.traversal.inference.Type.*
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

    private val enumTypes = mutableMapOf<AST.EnumDecl, Type>()

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
                    report(fn.rlt.position.nel(), Report.Severity.NOTE, SemanticNote.TypeOfFunction(type.toString()))
                }
            }
        }
    }
}
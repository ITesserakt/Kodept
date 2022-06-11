package ru.tesserakt.kodept.traversal

import arrow.core.NonEmptyList
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.identity
import arrow.core.nel
import arrow.core.padZip
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.Internal
import ru.tesserakt.kodept.error.CompilerCrash
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import kotlin.reflect.KClass

object ForeignFunctionResolver : Transformer<AST.ForeignFunctionDecl>() {
    override val type: KClass<AST.ForeignFunctionDecl> = AST.ForeignFunctionDecl::class

    private val functionList = mutableMapOf<String, MutableList<AST.ForeignFunctionDecl.ExportedFunction>>()

    fun exportFunction(f: (List<Any?>) -> Any?, name: String, params: List<KClass<*>>, returns: KClass<*>) {
        functionList.computeIfAbsent(name) { mutableListOf() } += AST.ForeignFunctionDecl.ExportedFunction(
            f,
            params,
            returns
        )
    }

    private val AST.ForeignFunctionDecl.safeParams
        get() = params.map {
            val ref = it.type as AST.ResolvedTypeReference
            ref to ref.referral as AST.ForeignStructDecl
        }

    context(ReportCollector, Filepath) @OptIn(Internal::class)
    override fun transform(node: AST.ForeignFunctionDecl): EagerEffect<UnrecoverableError, out AST.Node> =
        eagerEffect {
            val wrong = node.params.filterNot {
                it.type is AST.ResolvedTypeReference && (it.type as AST.ResolvedTypeReference).referral is AST.ForeignStructDecl
            }
            if (wrong.isNotEmpty())
                shift<Unit>(
                    UnrecoverableError(
                        NonEmptyList.fromListUnsafe(wrong).map { it.rlt.type.position },
                        Report.Severity.ERROR,
                        SemanticError.ForeignFunctionParametersTypeMismatch(node.name)
                    )
                )
            if (node.returns != null)
                if (node.returns !is AST.ResolvedTypeReference || (node.returns as AST.ResolvedTypeReference).referral !is AST.ForeignStructDecl)
                    shift<Unit>(
                        UnrecoverableError(
                            node.returns?.rlt?.position?.nel(),
                            Report.Severity.ERROR,
                            SemanticError.ForeignFunctionReturnType(node.name)
                        )
                    )

            val selected = functionList[node.descriptor].orEmpty()
                .filter { (node.returns == null && it.returns == Unit::class) || it.returns.qualifiedName == node.returns!!.type.name }
                .filter { function ->
                    val left = node.safeParams.map { it.second.relatedWith }
                    val right = function.params.map { it.qualifiedName }
                    left.padZip(right) { a, b -> a != null && b != null && a == b }.all(::identity)
                }

            val function = when (selected.size) {
                0 -> shift(
                    UnrecoverableError(
                        node.rlt.position.nel(),
                        Report.Severity.ERROR,
                        SemanticError.ForeignFunctionLinkage(node.name)
                    )
                )

                1 -> selected[0]
                else -> shift(
                    UnrecoverableError(
                        node.rlt.position.nel(),
                        Report.Severity.CRASH,
                        CompilerCrash("Multiple implementations found")
                    )
                )
            }

            node.copy(action = function)
        }
}

inline fun <reified R> ForeignFunctionResolver.exportFunction(fnRef: String, crossinline f: () -> R) =
    exportFunction({ f() }, fnRef, emptyList(), R::class)

inline fun <reified T1, reified R> ForeignFunctionResolver.exportFunction(
    fnRef: String,
    crossinline f: (T1) -> R,
) = exportFunction({ f(it[0] as T1) }, fnRef, listOf(T1::class), R::class)

inline fun <reified T1, reified T2, reified R> ForeignFunctionResolver.exportFunction(
    fnRef: String,
    crossinline f: (T1, T2) -> R,
) = exportFunction({ f(it[0] as T1, it[1] as T2) }, fnRef, listOf(T1::class, T2::class), R::class)
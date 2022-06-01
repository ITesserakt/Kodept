package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.identity
import arrow.core.nel
import arrow.core.nonEmptyListOf
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.core.walkDownTop
import ru.tesserakt.kodept.error.CompilerCrash
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import kotlin.reflect.KClass

object InitializationTransformer : Transformer<AST.Assignment>() {
    override val type: KClass<AST.Assignment> = AST.Assignment::class

    init {
        dependsOn(DereferenceTransformer)
    }

    context(ReportCollector, Filename) override fun transform(node: AST.Assignment): EagerEffect<UnrecoverableError, out AST.Node> =
        eagerEffect {
            when (val left = node.left) {
                is AST.Dereference -> node
                is AST.FunctionCall -> node
                is AST.ResolvedReference -> {
                    when (val referral = left.referral) {
                        is AST.FunctionDecl, is AST.Parameter, is AST.InferredParameter -> shift(
                            UnrecoverableError(
                                nonEmptyListOf(
                                    left.referral.rlt.position,
                                    node.rlt.position
                                ), Report.Severity.ERROR, SemanticError.ImmutableConstruct(left.referral.name)
                            )
                        )

                        is AST.InitializedVar -> if (!referral.mutable) shift<Unit>(
                            UnrecoverableError(
                                nonEmptyListOf(
                                    left.referral.rlt.position,
                                    node.rlt.position
                                ),
                                Report.Severity.ERROR, SemanticError.ImmutableVariable(referral.name)
                            )
                        )

                        is AST.VariableDecl -> referral.parent!!.replaceChild(
                            referral,
                            AST.InitializedVar(referral, node.right)
                        )
                    }
                    AST.Stub(node)
                }

                is AST.TypeReference -> node
                is AST.Reference -> shift<AST.Node>(
                    UnrecoverableError(
                        node.rlt.position.nel(),
                        Report.Severity.CRASH,
                        CompilerCrash("All references should be resolved")
                    )
                )
            }
        }
}

object VariableScope : Transformer<AST.VariableDecl>() {
    override val type: KClass<AST.VariableDecl> = AST.VariableDecl::class

    init {
        dependsOn(objectUniqueness)
    }

    context(ReportCollector, Filename) override fun transform(node: AST.VariableDecl): EagerEffect<UnrecoverableError, out AST.Node> {
        val nearestBlock = node.walkDownTop(::identity).filterIsInstance<AST.ExpressionList>().first()
        val varIndex = nearestBlock.expressions.indexOf(node)
        val (outer, inner) = nearestBlock.expressions.withIndex().partition { it.index < varIndex }
        val scope = AST.ExpressionList(inner.map { it.value })
        nearestBlock.parent!!.replaceChild(nearestBlock, AST.ExpressionList(outer.map { it.value } + scope))
        return eagerEffect { node }
    }
}
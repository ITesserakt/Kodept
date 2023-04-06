package ru.tesserakt.kodept.traversal

import arrow.core.raise.EagerEffect
import arrow.core.raise.eagerEffect
import arrow.core.nonEmptyListOf
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import kotlin.reflect.KClass

object InitializationTransformer : Transformer<AST.Assignment>() {
    override val type: KClass<AST.Assignment> = AST.Assignment::class

    init {
        dependsOn(ReferenceResolver)
    }

    context(ReportCollector, Filepath) override fun transform(node: AST.Assignment): EagerEffect<UnrecoverableError, out AST.Node> =
        eagerEffect {
            when (val left = node.left) {
                is AST.FunctionCall -> node
                is AST.ResolvedReference -> {
                    when (val referral = left.referral) {
                        is AST.FunctionDecl, is AST.Parameter, is AST.InferredParameter, is AST.ForeignFunctionDecl, is AST.AbstractFunctionDecl -> failWithReport(
                            nonEmptyListOf(
                                referral.rlt.position,
                                node.rlt.position
                            ), Report.Severity.ERROR, SemanticError.ImmutableConstruct(referral.name)
                        )

                        is AST.InitializedVar -> if (!referral.mutable) failWithReport(
                            nonEmptyListOf(
                                referral.rlt.position,
                                node.rlt.position
                            ),
                            Report.Severity.ERROR, SemanticError.ImmutableVariable(referral.name)
                        ) else node
                    }
                }

                is AST.TypeReference -> node
                is AST.Reference -> with(left) { ReferenceResolver.contract(this) }
            }
        }
}


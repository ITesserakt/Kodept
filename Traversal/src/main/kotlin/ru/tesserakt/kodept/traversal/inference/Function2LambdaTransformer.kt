package ru.tesserakt.kodept.traversal.inference

import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.core.move
import ru.tesserakt.kodept.core.new
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.traversal.Transformer
import ru.tesserakt.kodept.traversal.UnrecoverableError
import java.util.*
import kotlin.reflect.KClass

object Function2LambdaTransformer : Transformer<AST.FunctionDecl>() {
    override val type: KClass<AST.FunctionDecl> = AST.FunctionDecl::class

    context(ReportCollector, Filepath) override fun transform(node: AST.FunctionDecl) =
        eagerEffect<UnrecoverableError, _> {
            with(node.rlt) {
                val lambda = AST.LambdaExpr(node.params.new(), node.rest.move(), node.returns?.new()).withRLT()
                node.copy(restCell = lambda.move()).withRLT()
            }
        }
}
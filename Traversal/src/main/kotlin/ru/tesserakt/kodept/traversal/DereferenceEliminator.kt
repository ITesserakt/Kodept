package ru.tesserakt.kodept.traversal

import arrow.core.continuations.eagerEffect
import arrow.core.nel
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import kotlin.reflect.KClass

object DereferenceEliminator : Transformer<AST.Dereference>() {
    override val type: KClass<AST.Dereference> = AST.Dereference::class

    context(ReportCollector, Filepath) override fun transform(node: AST.Dereference) = eagerEffect {
        /**
         * readInt().someCallWithParams(2) =>
         * someCallWithParams(readInt(), 2)
         */

        val (additionalParams, ref) = when (val right = node.right) {
            is AST.FunctionCall -> right.params to right.reference
            is AST.Reference -> emptyList<AST.Expression>() to right
            is AST.TypeReference -> failWithReport(
                right.rlt.position.nel(),
                Report.Severity.ERROR,
                SemanticError.TypeInDereference(right.name)
            )
        }

        with(node.accessRLT<RLT.Access>()?.dot ?: node.rlt) {
            AST.FunctionCall(ref.move(), listOf(node.left.move()) + additionalParams.move()).withRLT()
        }
    }

    val contract = Contract<AST.Dereference> {
        "$this should not be in AST"
    }
}
package ru.tesserakt.kodept.traversal.inference

import arrow.core.continuations.eagerEffect
import arrow.core.nel
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import ru.tesserakt.kodept.error.SemanticNote
import ru.tesserakt.kodept.traversal.*

object TypeInferenceAnalyzer : Analyzer() {
    init {
        dependsOn(
            BinaryOperatorDesugaring,
            ReferenceResolver,
            TypeReferenceResolver,
            UnaryOperatorDesugaring
        )
    }

    override fun ReportCollector.analyze(ast: AST) = eagerEffect {
        var context = Assumptions.empty()

        ast.flatten().forEach { node ->
            if (node is AST.FunctionDecl) {
                val (lambda, ctxWithParams) = node.convert()
                val (_, type) = (lambda infer ctxWithParams.combine(context)).bind()
                val generalized = ctxWithParams.generalize(type)
                context = context.and(Language.Var(node.name), generalized)
                with(ast.filepath) {
                    report(
                        node.rlt.position.nel(),
                        Report.Severity.NOTE,
                        SemanticNote.TypeOfFunction("$lambda : $generalized")
                    )
                }
            }
        }
    }.handleErrorWith {
        eagerEffect {
            with(ast.filepath) {
                when (it) {
                    is Errors.CannotUnify -> failWithReport(
                        null,
                        Report.Severity.ERROR,
                        SemanticError.MismatchedType(it.type.toString(), it.with.toString())
                    )
                    is Errors.InfiniteType -> failWithReport(
                        null, Report.Severity.ERROR, SemanticError.InfiniteType(it.type.toString())
                    )
                    is Errors.UnknownVariable -> failWithReport(
                        null, Report.Severity.CRASH, SemanticError.ReferenceCannotBeTyped(it.variable.name)
                    )
                }
            }
        }
    }
}
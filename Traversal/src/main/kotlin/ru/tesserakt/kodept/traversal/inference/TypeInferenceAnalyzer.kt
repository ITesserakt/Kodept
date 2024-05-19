package ru.tesserakt.kodept.traversal.inference

import arrow.core.continuations.eagerEffect
import arrow.core.nel
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Tree
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

        ast.flatten(Tree.SearchMode.Preorder).forEach { node ->
            if (node is AST.FunctionDecl) {
                val (lambda, ctxWithParams) = node.convert()
                val bind = Language.Var("${node.name}\$${node.id}")
                val (ctx, type) = (lambda infer ctxWithParams
                    .combine(context)
                    .and(bind, MonomorphicType.Var())).bind()
                val generalized = ctxWithParams.combine(ctx).combine(context).generalize(type)
                context = context.and(bind, generalized)
                with(ast.filepath) {
                    report(
                        node.rlt.position.nel(),
                        Report.Severity.NOTE,
                        SemanticNote.TypeOfFunction(generalized.toString())
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
                        null, Report.Severity.ERROR, SemanticError.InfiniteType(it.type.toString(), it.with.toString())
                    )
                    is Errors.UnknownVariable -> failWithReport(
                        null, Report.Severity.CRASH, SemanticError.ReferenceCannotBeTyped(it.variable.name)
                    )
                }
            }
        }
    }
}
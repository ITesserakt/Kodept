package ru.tesserakt.kodept.traversal

import arrow.core.NonEmptyList
import arrow.core.continuations.eagerEffect
import arrow.core.nel
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.core.rlt
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticWarning

val typeSimplifier = object : Transformer<AST.TypeExpression> {
    override val type = AST.TypeExpression::class

    context(ReportCollector, Filename) override fun transform(node: AST.TypeExpression) =
        eagerEffect<UnrecoverableError, AST.Node> {
            when (node) {
                is AST.TupleType -> {
                    if (node.items.size == 1)
                        Report(
                            this@Filename, node.rlt.position.nel(), Report.Severity.WARNING,
                            SemanticWarning.AlignWithType(node.items.first().toString())
                        ).report()
                    node
                }
                is AST.Type -> node
                is AST.UnionType -> {
                    val (unique, repeating) = node.items.groupBy { it }.values.partition { it.size == 1 }
                    repeating.reportEach {
                        Report(
                            this@Filename, node.rlt.position.nel(), Report.Severity.WARNING,
                            SemanticWarning.NonUniqueUnionItems(node.toString())
                        )
                    }
                    node.items = NonEmptyList.fromListUnsafe((unique + repeating).map { it[0] })
                    node
                }
            }
        }
}
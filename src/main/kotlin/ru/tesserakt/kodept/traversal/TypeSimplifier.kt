package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffectScope
import arrow.core.continuations.eagerEffect
import arrow.core.nel
import arrow.core.nonEmptyListOf
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.core.rlt
import ru.tesserakt.kodept.core.wrap
import ru.tesserakt.kodept.error.CompilerCrash
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticWarning

object TypeSimplifier : Transformer<AST.TypeExpression> {
    override val type = AST.TypeExpression::class

    context (ReportCollector, Filename)
            private fun AST.TupleType.transformTuple() = if (items.size == 1) {
        Report(
            this@Filename, rlt.position.nel(), Report.Severity.WARNING,
            SemanticWarning.AlignWithType(items.first().toString())
        ).report()
        items.first()
    } else this

    context(AST.UnionType, ReportCollector, Filename)
            private suspend fun EagerEffectScope<UnrecoverableError>.transformUnion(): AST.TypeExpression {
        ensure(items.size >= 2) {
            UnrecoverableError(
                Report(
                    this@Filename,
                    this@UnionType.rlt.position.nel(),
                    Report.Severity.CRASH,
                    CompilerCrash("Union type behaves like ordinary type: $items")
                )
            )
        }

        items.filterIsInstance<AST.UnionType>().reportEach {
            Report(
                this@Filename,
                it.rlt.position.nel(),
                Report.Severity.WARNING,
                SemanticWarning.AlignWithType(it.toString())
            )
        }

        val flattenedItems = items.flatMap {
            when (it) {
                is AST.UnionType -> it.items
                else -> nonEmptyListOf(it)
            }
        }

        val (unique, repeating) = flattenedItems.groupBy { it }.values.partition { it.size == 1 }
        repeating.reportEach {
            Report(
                this@Filename, this@UnionType.rlt.position.nel(), Report.Severity.WARNING,
                SemanticWarning.NonUniqueUnionItems(this@UnionType.toString())
            )
        }
        val items = (unique + repeating).map { it[0] }
        return when (items.size) {
            0 -> shift(
                UnrecoverableError(
                    Report(
                        this@Filename,
                        this@UnionType.rlt.position.nel(),
                        Report.Severity.CRASH,
                        CompilerCrash("Search for repeating elements failed")
                    )
                )
            )

            1 -> items.first()
            else -> copy(_items = items.toMutableList()).also { it.metadata += this@UnionType.rlt.wrap() }
        }
    }

    context(ReportCollector, Filename) override fun transform(node: AST.TypeExpression) =
        eagerEffect<UnrecoverableError, AST.Node> {
            when (node) {
                is AST.TupleType -> node.transformTuple()
                is AST.Type -> node
                is AST.UnionType -> with(node) { transformUnion() }
            }
        }
}
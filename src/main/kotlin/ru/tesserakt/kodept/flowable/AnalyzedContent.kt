package ru.tesserakt.kodept.flowable

import arrow.core.IorNel
import arrow.core.flatMap
import arrow.typeclasses.Semigroup
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.FileRelative
import ru.tesserakt.kodept.core.mapWithFilename
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.traversal.unwrap

context (ru.tesserakt.kodept.CompilationContext)
class AnalyzedContent(flowable: Flowable.Data.ErroneousAST) : Flowable<AnalyzedContent.Data> {
    data class Data(override val ast: Sequence<FileRelative<IorNel<Report, AST>>>) : Flowable.Data.ErroneousAST

    private val analyzed = flowable.ast.mapWithFilename { result ->
        result.flatMap(Semigroup.nonEmptyList()) {
            analyzers.foldAST(it) { analyzer, acc ->
                unwrap { with(analyzer) { analyzeWithCaching(acc) }.map { acc } }
            }
        }
    }
    override val result = Data(analyzed)
}
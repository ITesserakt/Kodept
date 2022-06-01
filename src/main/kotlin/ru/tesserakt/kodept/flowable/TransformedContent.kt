package ru.tesserakt.kodept.flowable

import arrow.core.IorNel
import arrow.core.continuations.eagerEffect
import arrow.core.flatMap
import arrow.core.getOrHandle
import arrow.typeclasses.Semigroup
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.traversal.Analyzer
import ru.tesserakt.kodept.traversal.Transformer
import ru.tesserakt.kodept.traversal.transformOrSkip
import ru.tesserakt.kodept.traversal.unwrap

context (CompilationContext)
class TransformedContent(flowable: Flowable.Data.ErroneousAST) : Flowable<TransformedContent.Data> {
    data class Data(override val ast: Sequence<FileRelative<IorNel<Report, AST>>>) : Flowable.Data.ErroneousAST

    private val sorted = OrientedGraph.fromNodes(transformers + analyzers).sortedLayers()
        .getOrHandle { throw IllegalStateException("Found cycles in processors") }.flatten()

    private val transformed = flowable.ast.mapWithFilename { either ->
        either.flatMap(Semigroup.nonEmptyList()) { ast ->
            sorted.foldAST(ast) { value, acc ->
                when (value) {
                    is Transformer<*> -> executeTransformer(acc, value)
                    is Analyzer -> unwrap { with(value) { analyzeWithCaching(acc) }.map { acc } }
                }
            }
        }
    }

    private fun Filename.executeTransformer(
        acc: AST,
        transformer: Transformer<*>,
    ) = unwrap {
        eagerEffect {
            val (head, tail) = acc.flatten(mode = Tree.SearchMode.Postorder)
                .run { first { it.parent == null } to filter { it.parent != null } }
            tail.forEach {
                val parent = it.parent!!
                val transformed = transformer.transformOrSkip(it).bind()
                if (transformed != it)
                    parent.replaceChild(it, transformed)
            }
            val root = transformer.transformOrSkip(head).bind()
            AST(root, this@executeTransformer)
        }
    }

    override val result = Data(transformed)
}
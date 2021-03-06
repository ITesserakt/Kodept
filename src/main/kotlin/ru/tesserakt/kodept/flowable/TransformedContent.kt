package ru.tesserakt.kodept.flowable

import arrow.core.*
import arrow.core.continuations.eagerEffect
import arrow.typeclasses.Semigroup
import mu.KotlinLogging
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.traversal.Analyzer
import ru.tesserakt.kodept.traversal.Transformer
import ru.tesserakt.kodept.traversal.transformOrSkip
import ru.tesserakt.kodept.traversal.unwrap
import kotlin.collections.flatten

private val logger = KotlinLogging.logger("[Compiler]")

context (CompilationContext)
class TransformedContent(flowable: Flowable.Data.ErroneousAST) : Flowable<TransformedContent.Data> {
    data class Data(override val ast: Sequence<FileRelative<IorNel<Report, AST>>>) : Flowable.Data.ErroneousAST

    private fun <T> Either<OrientedGraph.Errors, T>.handleErrors() = getOrHandle {
        throw "Found errors in processors: ${
            when (it) {
                is OrientedGraph.Cycle<*> -> "cycle of:\n${it.inside.joinToString("\n")}"
                OrientedGraph.NotFound -> "common error"
            }
        }".let(::IllegalStateException)
    }

    private val sorted = OrientedGraph
        .fromNodes(analyzers + transformers)
        .sortedLayers()
        .handleErrors()
        .flatten()

    private val transformed = flowable.ast.mapWithFilename { either ->
        logger.info("Analyzing ${this.name}...")

        either.flatMap(Semigroup.nonEmptyList()) { ast ->
            sorted.foldAST(ast) { value, acc ->
                logger.trace("Executing $value")
                when (value) {
                    is Transformer<*> -> executeTransformer(acc, value)
                    is Analyzer -> unwrap { value.analyzeWithCaching(acc).map { acc } }
                }
            }
        }
    }

    private fun Filepath.executeTransformer(tree: AST, transformer: Transformer<*>) = unwrap {
        eagerEffect {
            val changes = tree
                .flatten(transformer.traverseMode)
                .traverse {
                    transformer.transformOrSkip(it).map { new ->
                        if (new != null && it != new) it to new
                        else null
                    }
                }.bind().filterNotNull()

            if (changes.isEmpty()) tree
            else tree.copyWith { changes.replaced() }
        }
    }

    override val result = Data(transformed)
}
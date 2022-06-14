package ru.tesserakt.kodept.flowable

import arrow.core.*
import arrow.core.continuations.eagerEffect
import arrow.typeclasses.Semigroup
import mu.KotlinLogging
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.CompilerCrash
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.traversal.*
import kotlin.collections.flatten

private val logger = KotlinLogging.logger("[Compiler]")

context (CompilationContext)
class TransformedContent(flowable: Flowable.Data.ErroneousAST) : Flowable<TransformedContent.Data> {
    data class Data(override val ast: Sequence<FileRelative<IorNel<Report, AST>>>) : Flowable.Data.ErroneousAST

    private val sorted = OrientedGraph.fromNodes(analyzers + transformers).sortedLayers()
        .getOrHandle { throw IllegalStateException("Found cycles in processors") }.flatten()

    private val transformed = flowable.ast.mapWithFilename { either ->
        logger.info("Analyzing ${this.name}...")

        either.flatMap(Semigroup.nonEmptyList()) { ast ->
            sorted.foldAST(ast) { value, acc ->
                logger.trace("Executing $value")
                when (value) {
                    is SpecificTransformer<*> -> executeTransformer(acc, value)
                    is Analyzer -> unwrap { value.analyzeWithCaching(acc).map { acc } }
                }
            }
        }
    }

    @OptIn(Internal::class)
    private fun Filepath.executeTransformer(acc: AST, transformer: SpecificTransformer<*>) = unwrap {
        val (head, tail) = acc.flatten(transformer.traverseMode).toList()
            .run { filterIsInstance<AST.NodeWithoutParent>().first() to filterIsInstance<AST.NodeWithParent>() }
        eagerEffect {
            tail.map { transformer.transformOrSkip(it).bind() }.forEach { (old, new) ->
                if (!(old === new || (old.parent as AST.NodeBase).replaceChild(old, new))) failWithReport(
                    nonEmptyListOf((old.parent as AST.NodeBase).rlt.position, new.rlt.position),
                    Report.Severity.CRASH,
                    CompilerCrash("After applying $transformer to $old the AST didn't change")
                )
            }
            val (_, newRoot) = transformer.transformOrSkip(head).bind()
            AST(newRoot as AST.NodeWithoutParent, this@executeTransformer)
        }
    }

    override val result = Data(transformed)
}
package ru.tesserakt.kodept.core.flowable

import arrow.core.IorNel
import arrow.core.continuations.eagerEffect
import arrow.core.flatMap
import arrow.typeclasses.Semigroup
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.traversal.transformOrSkip
import ru.tesserakt.kodept.traversal.unwrap

context (CompilationContext)
class TransformedContent(flowable: Flowable.Data.ErroneousAST) : Flowable<TransformedContent.Data> {
    data class Data(override val ast: Sequence<FileRelative<IorNel<Report, AST>>>) : Flowable.Data.ErroneousAST

    private val transformed = flowable.ast.mapWithFilename { either ->
        either.flatMap(Semigroup.nonEmptyList()) { ast ->
            transformers.foldAST(ast) { transformer, acc ->
                unwrap {
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
                        AST(root, this@mapWithFilename)
                    }
                }
            }
        }
    }
    override val result = Data(transformed)
}
package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

abstract class SpecificTransformer<A : AST.Node> : Depended() {
    abstract val type: KClass<A>

    context (ReportCollector, Filepath)
            abstract fun transformTo(node: A): EagerEffect<UnrecoverableError, Pair<AST.Node, AST.Node>>
}

abstract class Transformer<A : AST.Node> : SpecificTransformer<A>() {
    context (ReportCollector, Filepath)
            abstract fun transform(node: A): EagerEffect<UnrecoverableError, out AST.Node>

    context(ReportCollector, Filepath) final override fun transformTo(node: A) =
        transform(node).map { node as AST.Node to it }
}

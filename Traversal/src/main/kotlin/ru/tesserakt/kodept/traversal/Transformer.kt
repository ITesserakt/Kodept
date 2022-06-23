package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import arrow.core.identity
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.Tree
import ru.tesserakt.kodept.core.walkDownTop
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass
import kotlin.reflect.safeCast

abstract class Transformer<A : AST.Node> : Depended() {
    abstract val type: KClass<A>

    open val traverseMode = Tree.SearchMode.Postorder

    open fun filterCandidatesBy(candidate: AST.Node): A? = type.safeCast(candidate)

    protected inline fun <reified N : AST.Node> AST.NodeWithParent.getNearest() =
        walkDownTop(::identity).filterIsInstance<N>().first()

    context (ReportCollector, Filepath)
            abstract fun transform(node: A): EagerEffect<UnrecoverableError, out AST.Node>
}

package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

abstract class Transformer<A : AST.Node> : Depended() {
    abstract val type: KClass<A>

    context (ReportCollector, Filename)
            abstract fun transform(node: A): EagerEffect<UnrecoverableError, out AST.Node>
}
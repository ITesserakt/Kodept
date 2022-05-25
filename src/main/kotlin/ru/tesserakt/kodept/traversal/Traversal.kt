@file:Suppress("UNCHECKED_CAST")

package ru.tesserakt.kodept.traversal

import arrow.core.Ior
import arrow.core.NonEmptyList
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.leftIor
import arrow.core.rightIor
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

sealed interface ControlSwitching
object UnrecoverableError : ControlSwitching
object Skip : ControlSwitching

interface Transformer<A : AST.Node> {
    val type: KClass<A>

    context (ReportCollector)
    fun transform(node: A): EagerEffect<UnrecoverableError, AST.Node>
}

context (ReportCollector)
fun <A : AST.Node> Transformer<A>.skipOrTransform(node: AST.Node) =
    if (node::class == type) transform(node as A)
    else eagerEffect { node }

fun interface Analyzer {
    context(ReportCollector)
    fun analyze(ast: AST): EagerEffect<UnrecoverableError, Unit>
}

@Suppress("unused")
inline fun <T> unwrapNullable(f: ReportCollector.() -> EagerEffect<out ControlSwitching, out T>) =
    with(ReportCollector()) {
        f(this).fold({
            when (it) {
                Skip -> NonEmptyList.fromList(collectedReports).fold({ null.rightIor() }, { Ior.Both(it, null) })
                UnrecoverableError -> definitelyCollected.leftIor()
            }
        }, {
            NonEmptyList.fromList(collectedReports).fold({ it.rightIor() }, { list -> Ior.Both(list, it) })
        })
    }

fun <T> unwrap(f: ReportCollector.() -> EagerEffect<out UnrecoverableError, T>) = with(ReportCollector()) {
    f(this).fold({ definitelyCollected.leftIor() }, {
        if (collectedReports.isEmpty()) it.rightIor()
        else Ior.Both(definitelyCollected, it)
    })
}
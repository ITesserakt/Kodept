package ru.tesserakt.kodept.traversal

import arrow.core.Ior
import arrow.core.NonEmptyList
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.leftIor
import arrow.core.rightIor
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass
import kotlin.reflect.safeCast

sealed interface ControlSwitching
object UnrecoverableError : ControlSwitching
object Skip : ControlSwitching

interface Transformer<A : AST.Node> {
    val type: KClass<A>

    context (ReportCollector, Filename)
    fun transform(node: A): EagerEffect<UnrecoverableError, out AST.Node>
}

context (ReportCollector, Filename)
fun <A : AST.Node> Transformer<A>.skipOrTransform(node: AST.Node) =
    type.safeCast(node)?.let { transform(it) } ?: eagerEffect { node }

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
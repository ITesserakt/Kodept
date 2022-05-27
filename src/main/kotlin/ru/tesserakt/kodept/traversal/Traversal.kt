package ru.tesserakt.kodept.traversal

import arrow.core.Ior
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.leftIor
import arrow.core.nel
import arrow.core.rightIor
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass
import kotlin.reflect.safeCast

sealed interface ControlSwitching
data class UnrecoverableError(val crashReport: Report) : ControlSwitching
object Skip : ControlSwitching

interface Transformer<A : AST.Node> {
    val type: KClass<A>

    context (ReportCollector, Filename)
    fun transform(node: A): EagerEffect<UnrecoverableError, out AST.Node>
}

context (ReportCollector, Filename)
fun <A : AST.Node> Transformer<A>.transformOrSkip(node: AST.Node) =
    type.safeCast(node)?.let { transform(it) } ?: eagerEffect { node }

fun interface Analyzer {
    context(ReportCollector)
    fun analyze(ast: AST): EagerEffect<UnrecoverableError, Unit>
}

fun <T> unwrap(f: ReportCollector.() -> EagerEffect<out UnrecoverableError, T>) = with(ReportCollector()) {
    f(this).fold({ (it.crashReport.nel() + collectedReports).leftIor() }, {
        if (collectedReports.isEmpty()) it.rightIor()
        else if (collectedReports.any { report -> report.severity == Report.Severity.ERROR }) definitelyCollected.leftIor()
        else Ior.Both(definitelyCollected, it)
    })
}
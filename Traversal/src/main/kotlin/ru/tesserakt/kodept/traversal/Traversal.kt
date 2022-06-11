package ru.tesserakt.kodept.traversal

import arrow.core.*
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.CodePoint
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.ReportMessage
import kotlin.reflect.safeCast

data class UnrecoverableError(val crashReport: Report) {
    companion object {
        context (Filename)
                operator fun invoke(
            position: NonEmptyList<CodePoint>?,
            severity: Report.Severity,
            message: ReportMessage,
        ) =
            UnrecoverableError(Report(this@Filename, position, severity, message))
    }
}

context (ReportCollector, Filename)
fun <A : AST.Node> SpecificTransformer<A>.transformOrSkip(node: AST.Node) =
    type.safeCast(node)?.let { transformTo(it) } ?: eagerEffect { node to node }

fun <T> unwrap(f: ReportCollector.() -> EagerEffect<out UnrecoverableError, T>) = with(ReportCollector()) {
    f(this).fold({ (it.crashReport.nel() + collectedReports).leftIor() }, {
        if (!hasReports) it.rightIor()
        else if (hasErrors) definitelyCollected.leftIor()
        else Ior.Both(definitelyCollected, it)
    })
}
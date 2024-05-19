package ru.tesserakt.kodept.traversal

import arrow.core.*
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.EagerEffectScope
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.CodePoint
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.ReportMessage

@JvmInline
value class UnrecoverableError(val crashReport: Report) {
    companion object {
        context (Filepath)
                operator fun invoke(
            position: NonEmptyList<CodePoint>?,
            severity: Report.Severity,
            message: ReportMessage,
        ) = UnrecoverableError(Report(position, severity, message))
    }
}

context (ReportCollector, Filepath)
fun <A : AST.Node> Transformer<A>.transformOrSkip(node: AST.Node) = eagerEffect {
    filterCandidatesBy(node)?.let { transform(it) }?.bind()
}

fun <T> unwrap(f: ReportCollector.() -> EagerEffect<out UnrecoverableError, T>) = with(ReportCollector()) {
    f(this).fold({ (it.crashReport.nel() + collectedReports).leftIor() }, {
        if (!hasReports) it.rightIor()
        else if (hasErrors) definitelyCollected.leftIor()
        else Ior.Both(definitelyCollected, it)
    })
}

context (Filepath)
        suspend fun EagerEffectScope<UnrecoverableError>.failWithReport(
    point: NonEmptyList<CodePoint>?,
    severity: Report.Severity,
    message: ReportMessage,
): Nothing = shift(UnrecoverableError(point, severity, message))
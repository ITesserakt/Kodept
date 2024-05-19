package ru.tesserakt.kodept.traversal

import arrow.core.*
import arrow.core.raise.Raise
import arrow.core.raise.eagerEffect
import arrow.core.raise.fold
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
    filterCandidatesBy(node)?.let { transform(it) }?.fold({ raise(it) }, { it })
}

fun <T> unwrap(f: context(Raise<UnrecoverableError>) ReportCollector.() -> T): Ior<NonEmptyList<Report>, T> =
    with(ReportCollector()) {
        eagerEffect { f(this, this@with) }.fold({ (it.crashReport.nel() + collectedReports).leftIor() }, {
            if (!hasReports) it.rightIor()
            else if (hasErrors) definitelyCollected.leftIor()
            else Ior.Both(definitelyCollected, it)
        })
    }

context (Filepath, Raise<UnrecoverableError>)
fun failWithReport(
    point: NonEmptyList<CodePoint>?,
    severity: Report.Severity,
    message: ReportMessage,
): Nothing = raise(UnrecoverableError(point, severity, message))
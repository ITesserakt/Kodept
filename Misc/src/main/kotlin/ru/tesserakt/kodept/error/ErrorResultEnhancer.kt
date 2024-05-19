package ru.tesserakt.kodept.error

import arrow.core.*
import com.github.h0tk3y.betterParse.parser.*
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.toCodePoint
import ru.tesserakt.kodept.parser.RefinementError

private fun AlternativesFailure.expand(onlyLeaves: Boolean): NonEmptyList<ErrorResult> =
    if (!onlyLeaves) NonEmptyList.fromListUnsafe(errors).flatMap {
        if (it is AlternativesFailure) it.expand(false) else it.nel()
    } else {
        val alternatives = errors.filterIsInstance<AlternativesFailure>()
        (if (alternatives.isNotEmpty())
            NonEmptyList.fromListUnsafe(alternatives).flatMap { it.expand(true) }
        else
            NonEmptyList.fromListUnsafe(errors))
    }

private fun NonEmptyList<ErrorResult>.findSimilarMismatches() =
    filter { it is MismatchedToken || it is RefinementError }
        .groupBy {
            when (it) {
                is MismatchedToken -> it.found.toCodePoint()
                is RefinementError -> it.actual.toCodePoint()
                else -> throw IllegalStateException("Impossible")
            }
        }.values.map { tokens ->
            NonEmptyList.fromListUnsafe(tokens.distinctBy {
                when (it) {
                    is MismatchedToken -> it.expected.name
                    is RefinementError -> it.expected.name
                    else -> throw IllegalStateException("Impossible")
                }
            })
        }

context (ErrorResultConfig)
        private fun AlternativesFailure.expandFlatten(filename: Filepath): NonEmptyList<Report> {
    val expanded = expand(onlyLeaves)
    val reports =
        expanded.map {
            it.takeIf { it !is MismatchedToken && it !is UnexpectedEof && it !is RefinementError }?.toReport(filename)
                .orEmpty()
        }.flatten()
    val mismatchedOrRefined = expanded.findSimilarMismatches().map { mismatchedTokens ->
        val (found, expected) = mismatchedTokens.map {
            when (it) {
                is MismatchedToken -> (it.found to it.expected)
                is RefinementError -> (it.actual to it.expected)
                else -> throw IllegalStateException("Impossible")
            }
        }.unzip()
        Report(
            filename,
            found.head.toCodePoint().nel(),
            Report.Severity.ERROR,
            SyntaxError.MismatchedToken(expected, found.head)
        )
    }
    val eofReport =
        expanded.filterIsInstance<UnexpectedEof>().toNonEmptyListOrNull()?.let { unexpectedEoves ->
            Report(
                filename,
                null,
                Report.Severity.ERROR,
                SyntaxError.UnexpectedEOF(unexpectedEoves.map { it.expected })
            )
        }
    return NonEmptyList.fromListUnsafe(reports + mismatchedOrRefined + listOfNotNull(eofReport))
}

data class ErrorResultConfig(val onlyLeaves: Boolean)

context (ErrorResultConfig)
fun ErrorResult.toReport(filename: Filepath): NonEmptyList<Report> = when (this) {
    is UnexpectedEof -> Report(filename, null, Report.Severity.ERROR, SyntaxError.UnexpectedEOF(expected.nel())).nel()
    is MismatchedToken -> Report(
        filename,
        found.toCodePoint().nel(),
        Report.Severity.ERROR,
        SyntaxError.MismatchedToken(expected.nel(), found)
    ).nel()

    is NoMatchingToken -> Report(
        filename,
        tokenMismatch.toCodePoint().nel(),
        Report.Severity.ERROR,
        SyntaxError.UnknownToken(tokenMismatch)
    ).nel()

    is UnparsedRemainder -> Report(
        filename,
        startsWith.toCodePoint().nel(),
        Report.Severity.ERROR,
        SyntaxError.UnparsedRemainder
    ).nel()

    is AlternativesFailure -> expandFlatten(filename)
    is RefinementError -> Report(
        filename,
        actual.toCodePoint().nel(),
        Report.Severity.ERROR,
        SyntaxError.MismatchedToken(expected.nel(), actual)
    ).nel()

    else -> Report(filename, null, Report.Severity.ERROR, SyntaxError.Common(this)).nel()
}
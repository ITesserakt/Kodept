package ru.tesserakt.kodept.error

import arrow.core.NonEmptyList
import arrow.core.flatten
import arrow.core.nel
import com.github.h0tk3y.betterParse.parser.*
import ru.tesserakt.kodept.lexer.toCodePoint

private fun AlternativesFailure.expand(): NonEmptyList<ErrorResult> =
    NonEmptyList.fromListUnsafe(errors).flatMap { (it as? AlternativesFailure)?.expand() ?: it.nel() }

private fun NonEmptyList<ErrorResult>.findSimilarMismatches() = filterIsInstance<MismatchedToken>()
    .groupBy { it.expected to it.found.toCodePoint() }.values
    .map { tokens -> NonEmptyList.fromListUnsafe(tokens.distinctBy { it.expected.name }) }

fun ErrorResult.toReport(filename: String): NonEmptyList<Report> = when (this) {
    is UnexpectedEof -> Report(filename, null, Report.Severity.ERROR, SyntaxError.UnexpectedEOF(expected.nel())).nel()
    is MismatchedToken -> Report(filename,
        found.toCodePoint().nel(),
        Report.Severity.ERROR,
        SyntaxError.MismatchedToken(expected.nel(), found)).nel()
    is NoMatchingToken -> Report(filename,
        tokenMismatch.toCodePoint().nel(),
        Report.Severity.ERROR,
        SyntaxError.UnknownToken(tokenMismatch)).nel()
    is UnparsedRemainder -> Report(filename,
        startsWith.toCodePoint().nel(),
        Report.Severity.ERROR,
        SyntaxError.UnparsedRemainder).nel()
    is AlternativesFailure -> expandFlatten(filename)
    else -> Report(filename, null, Report.Severity.ERROR, SyntaxError.Common(this)).nel()
}

private fun AlternativesFailure.expandFlatten(filename: String): NonEmptyList<Report> {
    val expanded = expand()
    val reports =
        expanded.map { it.takeIf { it !is MismatchedToken && it !is UnexpectedEof }?.toReport(filename).orEmpty() }
            .flatten()
    val mismatchedReports = expanded.findSimilarMismatches().map { mismatchedTokens ->
        Report(filename,
            mismatchedTokens.head.found.toCodePoint().nel(),
            Report.Severity.ERROR,
            SyntaxError.MismatchedToken(mismatchedTokens.map { it.expected }, mismatchedTokens.head.found))
    }
    val eofReport =
        NonEmptyList.fromList(expanded.filterIsInstance<UnexpectedEof>()).orNull()?.let { unexpectedEoves ->
            Report(filename,
                null,
                Report.Severity.ERROR,
                SyntaxError.UnexpectedEOF(unexpectedEoves.map { it.expected }))
        }
    return NonEmptyList.fromListUnsafe(reports + mismatchedReports + listOfNotNull(eofReport))
}
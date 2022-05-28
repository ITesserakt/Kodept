package ru.tesserakt.kodept.core.flowable

import arrow.core.IorNel
import arrow.core.leftIor
import arrow.core.rightIor
import com.github.h0tk3y.betterParse.parser.ErrorResult
import com.github.h0tk3y.betterParse.parser.Parsed
import com.github.h0tk3y.betterParse.parser.tryParseToEnd
import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.FileRelative
import ru.tesserakt.kodept.core.mapWithFilename
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.toReport
import ru.tesserakt.kodept.parser.RLT

context (CompilationContext)
class PreParsedContent(flowable: Flowable.Data.Tokens) : Flowable<PreParsedContent.Data> {
    data class Data(override val rlt: Sequence<FileRelative<IorNel<Report, RLT>>>) : Flowable.Data.ErroneousRawTree

    override val result = Data(flowable.tokens.mapWithFilename {
        when (val parsed = rootParser.tryParseToEnd(it, 0)) {
            is Parsed -> parsed.value.rightIor()
            is ErrorResult -> parsed.toReport(this).leftIor()
        }
    })
}
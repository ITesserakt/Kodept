package ru.tesserakt.kodept.core.flowable

import com.github.h0tk3y.betterParse.lexer.TokenMatchesSequence
import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.FileRelative

context (CompilationContext)
class TokenContent(flowable: Flowable.Data.Holder) : Flowable<TokenContent.Data> {
    data class Data(override val tokens: Sequence<FileRelative<TokenMatchesSequence>>) : Flowable.Data.Tokens

    override val result = Data(flowable.holder
        .walkThoughAll { FileRelative(lexer.tokenize(it.allText), it.filename) })
}
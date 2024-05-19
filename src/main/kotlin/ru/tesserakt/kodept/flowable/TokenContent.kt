package ru.tesserakt.kodept.flowable

import com.github.h0tk3y.betterParse.lexer.TokenMatchesSequence
import io.github.oshai.kotlinlogging.KotlinLogging
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.FileRelative

private val logger = KotlinLogging.logger("[Compiler]")

context (CompilationContext)
class TokenContent(flowable: Flowable.Data.Holder) : Flowable<TokenContent.Data> {
    data class Data(override val tokens: Sequence<FileRelative<TokenMatchesSequence>>) : Flowable.Data.Tokens

    override val result = Data(flowable.holder.walkThoughAll {
        logger.info { "Parsing ${it.filename.name}..." }

        FileRelative(lexer.tokenize(it.allText), it.filename)
    })
}
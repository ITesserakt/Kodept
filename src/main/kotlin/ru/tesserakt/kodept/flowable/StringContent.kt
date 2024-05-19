package ru.tesserakt.kodept.flowable

import arrow.eval.Eval
import io.github.oshai.kotlinlogging.KotlinLogging
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.CodeSource
import ru.tesserakt.kodept.core.ProgramCodeHolder
import ru.tesserakt.kodept.core.withFilename
import java.io.Reader

private val logger = KotlinLogging.logger("[Compiler]")

context(CompilationContext)
class StringContent : Flowable<StringContent.Data> {
    data class Data(
        override val source: Sequence<CodeSource>,
        override val holder: ProgramCodeHolder,
    ) : Flowable.Data.Source, Flowable.Data.Holder

    private val sources = loader.getSources()
    private val text = sources.map {
        it.withFilename {
            Eval.later {
                logger.info { "Loading ${it.name}..." }
                contents.bufferedReader().use(Reader::readText)
            }
        }
    }
    private val holder = ProgramCodeHolder(text.associate { it.filepath to it.value })

    override val result = Data(sources, holder)
}
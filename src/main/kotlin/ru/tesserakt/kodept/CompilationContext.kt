package ru.tesserakt.kodept

import arrow.core.rightIor
import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.lexer.Tokenizer
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.core.FileRelative
import ru.tesserakt.kodept.core.Loader
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.mapWithFilename
import ru.tesserakt.kodept.error.ErrorResultConfig
import ru.tesserakt.kodept.flowable.*
import ru.tesserakt.kodept.lexer.Lexer
import ru.tesserakt.kodept.parser.FileGrammar
import ru.tesserakt.kodept.traversal.Analyzer
import ru.tesserakt.kodept.traversal.Transformer

class CompilationContext private constructor(
    val loader: Loader,
    val lexer: Tokenizer,
    val rootParser: Parser<RLT>,
    val transformers: Set<Transformer<*>>,
    val analyzers: Set<Analyzer>,
) {
    class Builder {
        var lexer: Tokenizer = Lexer()
        lateinit var loader: Loader
        var rootParser = FileGrammar.map { RLT(it) }
        var transformers = setOf<Transformer<*>>()
        var analyzers = setOf<Analyzer>()

        fun build() = CompilationContext(loader, lexer, rootParser, transformers, analyzers)
    }

    @Suppress("unused")
    inner class Scope {
        fun <T : Flowable.Data, F : Flowable<T>> F.bind() = result
        fun <U : Flowable.Data, T : Flowable.Data, FT : Flowable<T>, FU : Flowable<U>> FT.then(f: T.() -> FU) =
            result.f()

        fun <T : Flowable.Data, FT : Flowable<T>, U> FT.also(f: T.() -> U) = result to result.f()

        context (Flowable.Data.Holder)
        fun <U : Flowable.Data.UnprocessedAST, FU : Flowable<U>, T : Flowable.Data.Forest, FT : Flowable<T>> FU.fallback(
            f: FT,
        ): Flowable<Flowable.Data.ErroneousAST> {
            val unboxed = f.bind()
            val ready = bind().unprocessedAST.mapWithFilename {
                when (it) {
                    null -> unboxed.forest.value()[this] ?: throw IllegalStateException("Unknown file passed: $this")
                    else -> it.rightIor()
                }
            }
            return object : Flowable<Flowable.Data.ErroneousAST> {
                override val result = object : Flowable.Data.ErroneousAST {
                    override val ast = ready
                }
            }
        }

        fun <U : Flowable.Data.UnprocessedAST, FU : Flowable<U>> FU.onlyGoodFiles() =
            object : Flowable<Flowable.Data.ErroneousAST> {
                override val result = object : Flowable.Data.ErroneousAST {
                    override val ast = this@onlyGoodFiles.result.unprocessedAST.mapNotNull {
                        it.value?.run { FileRelative(rightIor(), it.filename) }
                    }
                }
            }

        fun readSources() = StringContent()
        fun Flowable.Data.Holder.tokenize() = TokenContent(this)
        fun Flowable.Data.Tokens.parse(onlyLatestErrors: Boolean = true) =
            PreParsedContent(ErrorResultConfig(onlyLatestErrors), this)

        fun Flowable.Data.ErroneousRawTree.dropUnusedInfo() = ParsedContent(this)
        fun Flowable.Data.Source.retrieveFromCache() = HintASTContent(this)
        fun Flowable.Data.ErroneousAST.analyze() = TransformedContent(this)
    }

    inline infix fun <T> flow(scope: Scope.() -> T) = Scope().run(scope)

    companion object {
        operator fun invoke(block: Builder.() -> Unit = {}) =
            Builder().apply(block).build()
    }
}
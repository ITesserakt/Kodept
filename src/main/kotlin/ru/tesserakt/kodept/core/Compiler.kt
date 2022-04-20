package ru.tesserakt.kodept.core

import arrow.core.zip
import com.github.h0tk3y.betterParse.lexer.Tokenizer
import com.github.h0tk3y.betterParse.parser.*
import com.google.gson.GsonBuilder
import ru.tesserakt.kodept.lexer.Lexer
import ru.tesserakt.kodept.parser.FileGrammar
import java.io.Reader

class Compiler private constructor(
    private val loader: Loader,
    private val lexer: Tokenizer,
    private val rootParser: Parser<AST.Node>,
) {
    class Builder {
        var lexer: Tokenizer = Lexer()
        lateinit var loader: Loader
        var rootParser: Parser<AST.Node> = FileGrammar

        fun build() = Compiler(loader, lexer, rootParser)
    }

    private val sources by lazy {
        loader.getSources()
    }

    private val tokens by lazy {
        sources.map {
            val input = it.contents.bufferedReader().use(Reader::readText)
            lexer.tokenize(input) to input
        }
    }

    private val ast by lazy {
        fun <T, U> ParseResult<T>.map(f: (T) -> U) = when (this) {
            is Parsed -> object : Parsed<U>() {
                override val nextPosition: Int = this@map.nextPosition
                override val value: U = f(this@map.value)
            }
            is ErrorResult -> this
        }

        sources.zip(tokens).map { (source, tokens) ->
            when (val hint = source.hint) {
                null -> rootParser.tryParseToEnd(tokens.first, 0).map { AST(it, source.name) }
                else -> object : Parsed<AST>() {
                    override val nextPosition: Int = 0
                    override val value: AST = hint
                }
            }
        }
    }

    fun acquireContents() = sources

    fun tokenize() = tokens.map { it.first }

    fun parse() = ast

    private val gson = GsonBuilder()
        .setPrettyPrinting()
        .create()

    fun cache(with: (filename: String) -> Cache) =
        sources.zip(ast, tokens) { source, parsed, (_, text) ->
            CacheData(source.name, text, parsed.toParsedOrThrow().value)
        }.map {
            val cache = with(it.sourceName)
            cache.stream.writer().use { writer ->
                gson.toJson(gson.toJsonTree(it), gson.newJsonWriter(writer))
            }
            cache
        }

    companion object {
        operator fun invoke(loader: Loader, block: Builder.() -> Unit = {}) = Builder().apply(block).apply {
            this.loader = loader
        }.build()
    }
}
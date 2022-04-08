package ru.tesserakt.kodept

import com.github.h0tk3y.betterParse.lexer.Tokenizer
import com.github.h0tk3y.betterParse.parser.*
import com.google.gson.GsonBuilder
import ru.tesserakt.kodept.lexer.Lexer
import ru.tesserakt.kodept.parser.AST
import ru.tesserakt.kodept.parser.FileGrammar

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
        sources.map { lexer.tokenize(it.getContents().bufferedReader().readText()) }
    }

    private val ast by lazy {
        tokens.map {
            when (val result = rootParser.tryParseToEnd(it, 0)) {
                is Parsed -> object : Parsed<AST>() {
                    override val nextPosition: Int = result.nextPosition
                    override val value: AST = AST(result.value)
                }
                is ErrorResult -> result
            }
        }
    }

    fun acquireContents() = sources

    fun tokenize() = tokens

    fun parse() = ast

    private val gson = GsonBuilder()
        .setPrettyPrinting()
        .create()

    private fun Cache.runOn(tree: AST) = gson.toJson(gson.toJsonTree(tree), gson.newJsonWriter(stream.writer()))

    fun cache(with: (filename: String) -> Cache) = ast
        .zip(sources)
        .map { (result, src) ->
            result.toParsedOrThrow().value to src.name
        }.map { with(it.second).apply { runOn(it.first) } }

    companion object {
        operator fun invoke(loader: Loader, block: Builder.() -> Unit = {}) = Builder().apply(block).apply {
            this.loader = loader
        }.build()
    }
}
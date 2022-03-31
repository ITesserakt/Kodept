package ru.tesserakt.kodept

import com.github.h0tk3y.betterParse.lexer.Tokenizer
import com.github.h0tk3y.betterParse.parser.ErrorResult
import com.github.h0tk3y.betterParse.parser.Parsed
import com.github.h0tk3y.betterParse.parser.Parser
import com.github.h0tk3y.betterParse.parser.tryParseToEnd
import ru.tesserakt.kodept.lexer.Lexer
import ru.tesserakt.kodept.parser.AST
import ru.tesserakt.kodept.parser.FileGrammar

class Compiler private constructor(
    private val loader: Loader,
    private val lexer: Tokenizer,
    private val rootParser: Parser<AST.Node>
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

    fun acquireContents() = sources

    fun tokenize() = tokens

    fun parse() = tokens.map {
        when (val result = rootParser.tryParseToEnd(it, 0)) {
            is Parsed -> object : Parsed<AST>() {
                override val nextPosition: Int = result.nextPosition
                override val value: AST = AST(result.value)
            }
            is ErrorResult -> result
        }
    }

    companion object {
        operator fun invoke(block: Builder.() -> Unit) = Builder().apply(block).build()
    }
}
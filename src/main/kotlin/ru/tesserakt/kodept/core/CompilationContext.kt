package ru.tesserakt.kodept.core

import com.github.h0tk3y.betterParse.lexer.Tokenizer
import com.github.h0tk3y.betterParse.parser.Parser
import ru.tesserakt.kodept.analyzer.Analyzer
import ru.tesserakt.kodept.lexer.Lexer
import ru.tesserakt.kodept.parser.FileGrammar
import ru.tesserakt.kodept.transformer.Transformer

class CompilationContext private constructor(
    val loader: Loader,
    val lexer: Tokenizer,
    val rootParser: Parser<AST.Node>,
    val transformers: List<() -> Transformer>,
    val analyzers: List<Analyzer>,
) {
    class Builder {
        var lexer: Tokenizer = Lexer()
        lateinit var loader: Loader
        var rootParser: Parser<AST.Node> = FileGrammar
        var transformers = listOf<() -> Transformer>()
        var analyzers = listOf<Analyzer>()

        fun build() = CompilationContext(loader, lexer, rootParser, transformers, analyzers)
    }

    infix fun <T> flow(scope: CompilationContext.() -> Flowable<T>) = this.scope().result

    companion object {
        operator fun invoke(loader: Loader, block: Builder.() -> Unit = {}) =
            Builder().apply(block).apply {
                this.loader = loader
            }.build()
    }
}
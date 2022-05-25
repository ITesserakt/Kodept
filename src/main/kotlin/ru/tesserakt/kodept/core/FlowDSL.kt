package ru.tesserakt.kodept.core

import arrow.core.*
import arrow.core.continuations.eagerEffect
import arrow.typeclasses.Semigroup
import com.github.h0tk3y.betterParse.lexer.TokenMatchesSequence
import com.github.h0tk3y.betterParse.parser.tryParseToEnd
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.toReport
import ru.tesserakt.kodept.traversal.skipOrTransform
import ru.tesserakt.kodept.traversal.unwrap
import java.io.Reader

typealias ParseResult = IorNel<Report, AST>

private inline fun <T> Iterable<T>.foldAST(ast: AST, f: (T, AST) -> ParseResult): ParseResult =
    fold(ast.rightIor() as ParseResult) { acc, next ->
        f(next, acc.orNull()!!)
    }

@Suppress("NOTHING_TO_INLINE")
private inline fun <A, B> Either<A, B>.toIor() = fold({ it.leftIor() }) { it.rightIor() }

interface Flowable<T : Flowable.Data> {
    val result: T

    interface Data {
        interface Source : Data {
            val source: Sequence<CodeSource>
        }

        interface Holder : Data {
            val holder: ProgramCodeHolder
        }

        interface Tokens : Data {
            val tokens: Sequence<FileRelative<TokenMatchesSequence>>
        }

        interface Forest : Data {
            val forest: Eval<Map<Filename, ParseResult>>
        }

        interface UnprocessedAST : Data {
            val unprocessedAST: Sequence<FileRelative<AST?>>
        }

        interface ErroneousAST : Data {
            val ast: Sequence<FileRelative<ParseResult>>
        }
    }
}

context(CompilationContext)
class StringContent : Flowable<StringContent.Data> {
    data class Data(
        override val source: Sequence<CodeSource>,
        override val holder: ProgramCodeHolder,
    ) : Flowable.Data.Source, Flowable.Data.Holder

    private val sources = loader.getSources()
    private val text = sources.map {
        it.withFilename { Eval.later { contents.bufferedReader().use(Reader::readText) } }
    }
    private val holder = ProgramCodeHolder(text.associate { it.filename to it.value })

    override val result = Data(sources, holder)
}

context (CompilationContext)
class TokenContent(flowable: Flowable.Data.Holder) : Flowable<TokenContent.Data> {
    data class Data(override val tokens: Sequence<FileRelative<TokenMatchesSequence>>) : Flowable.Data.Tokens

    override val result = Data(flowable.holder
        .walkThoughAll { FileRelative(lexer.tokenize(it.allText), it.filename) })
}

context (CompilationContext)
class ParsedContent(flowable: Flowable.Data.Tokens) : Flowable<ParsedContent.Data> {
    data class Data(
        override val forest: Eval<Map<Filename, ParseResult>>,
        override val ast: Sequence<FileRelative<IorNel<Report, AST>>>,
    ) : Flowable.Data.ErroneousAST, Flowable.Data.Forest

    private val trees = flowable.tokens.mapWithFilename {
        rootParser
            .tryParseToEnd(it, 0).toEither().map { node -> AST(node, this) }
            .mapLeft { res -> res.toReport(this) }.toIor()
    }
    private val forest = Eval.later {
        trees.associate { it.filename to it.value }
    }

    override val result = Data(forest, trees)
}

class HintASTContent(a: Flowable.Data.Source) : Flowable<HintASTContent.Data> {
    data class Data(
        override val unprocessedAST: Sequence<FileRelative<AST?>>,
    ) : Flowable.Data.UnprocessedAST

    private val unprocessedAST = a.source.map { it.withFilename { hint } }

    override val result = Data(unprocessedAST)
}

context (CompilationContext)
class TransformedContent(flowable: Flowable.Data.ErroneousAST) : Flowable<TransformedContent.Data> {
    data class Data(override val ast: Sequence<FileRelative<IorNel<Report, AST>>>) : Flowable.Data.ErroneousAST

    private val transformed = flowable.ast.mapWithFilename { either ->
        either.flatMap(Semigroup.nonEmptyList()) { ast ->
            transformers.foldAST(ast) { ctor, acc ->
                val transformer = ctor()
                unwrap {
                    eagerEffect {
                        val head = acc.walkThrough { transformer.skipOrTransform(it) }.first().bind()
                        AST(head, this@mapWithFilename)
                    }
                }
            }
        }
    }
    override val result = Data(transformed)
}

context (CompilationContext)
class AnalyzedContent(flowable: Flowable.Data.ErroneousAST) : Flowable<AnalyzedContent.Data> {
    data class Data(override val ast: Sequence<FileRelative<IorNel<Report, AST>>>) : Flowable.Data.ErroneousAST

    private val analyzed = flowable.ast.mapWithFilename {
        it.flatMap(Semigroup.nonEmptyList()) {
            analyzers.foldAST(it) { analyzer, acc ->
                unwrap { analyzer.analyze(acc).map { acc } }
            }
        }
    }
    override val result = Data(analyzed)
}
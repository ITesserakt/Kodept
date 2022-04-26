@file:Suppress("FunctionName")

package ru.tesserakt.kodept.core

import arrow.core.*
import arrow.typeclasses.Semigroup
import com.github.h0tk3y.betterParse.lexer.TokenMatchesSequence
import com.github.h0tk3y.betterParse.parser.ErrorResult
import com.github.h0tk3y.betterParse.parser.tryParseToEnd
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.UnrecoverableError
import ru.tesserakt.kodept.error.toReport
import ru.tesserakt.kodept.transformer.acceptTransform
import java.io.Reader

interface Flowable<T> {
    val result: Sequence<T>
}

context(CompilationContext)
class StringContent : Flowable<CodeSource> {
    val sources = loader.getSources()

    val text = sources.map {
        it.withFilename { contents.bufferedReader().use(Reader::readText) }
    }

    override val result get() = sources
}

context (CompilationContext, StringContent)
class TokenContent : Flowable<FileRelative<TokenMatchesSequence>> {
    val tokens = text.mapWithFilename { lexer.tokenize(it) }
    override val result get() = tokens
}

typealias ParseResult<T> = Either<ErrorResult, T>

context (CompilationContext, TokenContent)
class ParsedContent : Flowable<FileRelative<ParseResult<AST>>> {
    val trees = tokens.mapWithFilename {
        rootParser.tryParseToEnd(it, 0).toEither().map { node -> AST(node, this) }
    }

    val forest by lazy {
        trees.mapWithFilename { Eval.now(it) }.associateBy { it.filename }
    }

    override val result get() = trees
}

context (StringContent, ParsedContent)
class HintASTContent : Flowable<FileRelative<ParseResult<AST>>> {
    val unprocessedAST by lazy {
        sources.map { it.withFilename { hint } }
    }

    val ast = unprocessedAST.map { relative ->
        if (relative.value == null)
            forest[relative.filename]!!.map { it.value() }
        else
            relative.map { it!!.right() }
    }

    override val result get() = ast
}

private inline fun <T> Iterable<T>.foldAST(ast: AST, f: MutableList<Report>.(T, AST) -> AST): IorNel<Report, AST> {
    val reports = mutableListOf<Report>()
    var newAST = ast
    for (item in this) {
        newAST = reports.f(item, newAST)
    }
    return when (val reportsNel = NonEmptyList.fromList(reports)) {
        None -> newAST.rightIor()
        is Some -> Ior.Both(reportsNel.value, newAST)
    }
}

context (CompilationContext)
class TransformedContent(astFlowable: Flowable<FileRelative<ParseResult<AST>>>) :
    Flowable<FileRelative<IorNel<Report, AST>>> {
    val transformed = astFlowable.result.mapWithFilename { either ->
        either.mapLeft { it.toReport(this) }
            .fold({ it.leftIor() }, { it.rightIor() })
            .flatMap(Semigroup.nonEmptyList()) { ast ->
                transformers.foldAST(ast) { ctor, acc ->
                    val transformer = ctor()
                    try {
                        AST(acc.root.acceptTransform(transformer), this@mapWithFilename).also {
                            this += transformer.collectedReports
                        }
                    } catch (e: UnrecoverableError) {
                        this@foldAST += transformer.collectedReports
                        return@flatMap NonEmptyList.fromListUnsafe(this@foldAST).leftIor()
                    }
                }
            }
    }
    override val result get() = transformed
}

context (CompilationContext, TransformedContent)
class AnalyzedContent : Flowable<FileRelative<IorNel<Report, AST>>> {
    val analyzed = transformed.mapWithFilename {
        it.flatMap(Semigroup.nonEmptyList()) {
            analyzers.foldAST(it) { analyzer, acc ->
                try {
                    analyzer.analyzeIndependently(acc).also { this += analyzer.collectedReports }
                    acc
                } catch (e: UnrecoverableError) {
                    this += analyzer.collectedReports
                    return@flatMap NonEmptyList.fromListUnsafe(this).leftIor()
                }
            }
        }
    }
    override val result get() = analyzed
}

context (CompilationContext)
fun acquireContent() = StringContent()

context (CompilationContext)
fun StringContent.tokenize() = TokenContent()

context (CompilationContext)
fun TokenContent.parse() = ParsedContent()

context (StringContent)
fun ParsedContent.withCache() = HintASTContent()

context (CompilationContext)
fun ParsedContent.transform() = TransformedContent(this)

context (CompilationContext)
fun HintASTContent.transform() = TransformedContent(this)

context (CompilationContext)
fun TransformedContent.analyze() = AnalyzedContent()
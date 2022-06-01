package ru.tesserakt.kodept.flowable

import arrow.core.Eval
import arrow.core.IorNel
import arrow.core.flatMap
import arrow.core.rightIor
import arrow.typeclasses.Semigroup
import com.github.h0tk3y.betterParse.lexer.TokenMatchesSequence
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.Report

typealias ParseResult = IorNel<Report, AST>

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

        interface ErroneousRawTree : Data {
            val rlt: Sequence<FileRelative<IorNel<Report, RLT>>>
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

internal inline fun <T> Iterable<T>.foldAST(ast: AST, f: (T, AST) -> ParseResult): ParseResult =
    fold(ast.rightIor() as ParseResult) { acc: ParseResult, t: T ->
        acc.flatMap(Semigroup.nonEmptyList()) { f(t, it) }
    }
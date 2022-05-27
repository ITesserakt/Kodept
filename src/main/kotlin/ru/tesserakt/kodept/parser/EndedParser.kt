package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.and
import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.combinators.or
import com.github.h0tk3y.betterParse.lexer.TokenMatchesSequence
import com.github.h0tk3y.betterParse.parser.*
import com.github.h0tk3y.betterParse.utils.Tuple2
import com.github.h0tk3y.betterParse.utils.Tuple3
import com.github.h0tk3y.betterParse.utils.Tuple4
import com.github.h0tk3y.betterParse.utils.Tuple5
import ru.tesserakt.kodept.lexer.ExpressionToken

class EndedParser<T, U>(
    private val elementParser: Parser<T>,
    private val separator: Parser<*>,
    private val end: Parser<U>,
    private val atLeast: Int,
    private val atMost: Int,
) : Parser<Tuple2<List<T>, U>> {
    init {
        require(atLeast >= 0) { "atLeast = $atLeast, expected non-negative" }
        require(atMost == -1 || atMost >= atLeast) { "atMost = $atMost is invalid, should be greater or equal than atLeast = $atLeast" }
    }

    private fun List<T>.checkEnd(tokens: TokenMatchesSequence, nextPosition: Int, failure: ErrorResult) =
        when (val ended = end.tryParse(tokens, nextPosition)) {
            is Parsed -> if (size >= atLeast)
                parsed(Tuple2(this, ended.value), ended.nextPosition)
            else failure

            is ErrorResult -> AlternativesFailure(listOf(failure, ended))
        }

    override fun tryParse(tokens: TokenMatchesSequence, fromPosition: Int): ParseResult<Tuple2<List<T>, U>> {
        val results = arrayListOf<T>()
        var nextPosition = fromPosition
        while (atMost == -1 || results.size < atMost) {
            when (val element = elementParser.tryParse(tokens, nextPosition)) {
                is ErrorResult -> return results.checkEnd(tokens, nextPosition, element)

                is Parsed -> {
                    nextPosition = element.nextPosition
                    results.add(element.value)
                    when (val sep = separator.tryParse(tokens, nextPosition)) {
                        is Parsed -> nextPosition = sep.nextPosition
                        is ErrorResult -> return results.checkEnd(tokens, nextPosition, sep)
                    }
                }
            }
        }
        return end.tryParse(tokens, nextPosition).map { Tuple2(results, it) }
    }
}

class UntilEOFParser<T>(
    private val elementParser: Parser<T>,
    private val separator: Parser<*>,
    private val atLeast: Int,
    private val atMost: Int,
) : Parser<List<T>> {
    private fun List<T>.checkEOF(
        tokens: TokenMatchesSequence,
        nextPosition: Int,
        maxPosition: Int,
        failure: ErrorResult,
    ) =
        if (size >= atLeast && (failure is UnexpectedEof || maxPosition == nextPosition))
            parsed(this, nextPosition)
        else failure

    override fun tryParse(tokens: TokenMatchesSequence, fromPosition: Int): ParseResult<List<T>> {
        val maxPosition = tokens.last().nextPosition
        val results = arrayListOf<T>()
        var nextPosition = fromPosition
        while (atMost == -1 || results.size < atMost) {
            when (val element = elementParser.tryParse(tokens, nextPosition)) {
                is ErrorResult -> return results.checkEOF(tokens, nextPosition, maxPosition, element)
                is Parsed -> {
                    nextPosition = element.nextPosition
                    results.add(element.value)
                    when (val sep = separator.tryParse(tokens, nextPosition)) {
                        is Parsed -> nextPosition = sep.nextPosition
                        is ErrorResult -> return results.checkEOF(tokens, nextPosition, maxPosition, sep)
                    }
                }
            }
        }
        return if (maxPosition == nextPosition)
            parsed(results, nextPosition)
        else UnparsedRemainder(tokens[nextPosition]!!.value)
    }
}

fun <T> trailingUntilEOF(
    elementParser: Parser<T>,
    separator: Parser<*> = ExpressionToken.SEMICOLON or ExpressionToken.NEWLINE,
    atLeast: Int = 0,
    atMost: Int = -1,
): Parser<List<T>> =
    UntilEOFParser(elementParser, separator, atLeast, atMost)

fun <T, U> Parser<T>.endsWith(
    end: Parser<U>,
    atMost: Int = -1,
    atLeast: Int = 0,
): Parser<Tuple2<List<T>, U>> = EndedParser(this, EmptyParser, end, atLeast, atMost)

fun <T> trailing(
    other: Parser<T>,
    separator: Parser<*> = ExpressionToken.SEMICOLON or ExpressionToken.NEWLINE,
    atLeast: Int = 0,
    atMost: Int = -1,
) = EndedParser(other, separator, EmptyParser, atLeast, atMost).map { it.t1 }

fun <T> strictTrailing(
    elementParser: Parser<T>,
    separator: Parser<*> = ExpressionToken.SEMICOLON or ExpressionToken.NEWLINE,
    atLeast: Int = 0,
    atMost: Int = -1,
) = Tuple4(elementParser, separator, atLeast, atMost)

operator fun <T, U> Tuple4<Parser<T>, Parser<*>, Int, Int>.times(end: Parser<U>) = EndedParser(t1, t2, end, t3, t4)

inline infix fun <reified T, U, V> Parser<T>.and(other: Parser<Tuple2<U, V>>) =
    this.and(other).map { Tuple3(it.t1, it.t2.t1, it.t2.t2) }

@JvmName("andT1T2")
inline infix fun <reified T1, reified T2, U, V> Parser<Tuple2<T1, T2>>.and(other: Parser<Tuple2<U, V>>) =
    this.and(other).map { Tuple4(it.t1.t1, it.t1.t2, it.t2.t1, it.t2.t2) }

@JvmName("andT1T2T3")
inline infix fun <reified T1, reified T2, reified T3, U, V> Parser<Tuple3<T1, T2, T3>>.and(other: Parser<Tuple2<U, V>>) =
    this.and(other).map { Tuple5(it.t1.t1, it.t1.t2, it.t1.t3, it.t2.t1, it.t2.t2) }
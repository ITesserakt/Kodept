package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.lexer.TokenMatchesSequence
import com.github.h0tk3y.betterParse.parser.ErrorResult
import com.github.h0tk3y.betterParse.parser.ParseResult
import com.github.h0tk3y.betterParse.parser.Parsed
import com.github.h0tk3y.betterParse.parser.Parser
import com.github.h0tk3y.betterParse.utils.Tuple2

class OptionalWithoutStartingParser<A, B>(
    private val startParser: Parser<A>,
    private val restParser: Parser<B>,
) : Parser<Tuple2<A, B>?> {
    override fun tryParse(tokens: TokenMatchesSequence, fromPosition: Int): ParseResult<Tuple2<A, B>?> {
        return when (val start = startParser.tryParse(tokens, fromPosition)) {
            is Parsed -> restParser.tryParse(tokens, start.nextPosition).map { Tuple2(start.value, it) }
            is ErrorResult -> parsed(null, fromPosition)
        }
    }
}

fun <A, B> optionalWithStart(startParser: Parser<A>, restParser: Parser<B>): Parser<Tuple2<A, B>?> =
    OptionalWithoutStartingParser(startParser, restParser)
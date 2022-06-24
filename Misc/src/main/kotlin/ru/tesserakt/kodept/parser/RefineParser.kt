package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.lexer.Token
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import com.github.h0tk3y.betterParse.lexer.TokenMatchesSequence
import com.github.h0tk3y.betterParse.lexer.noneMatched
import com.github.h0tk3y.betterParse.parser.*
import ru.tesserakt.kodept.lexer.ExpressionToken

data class RefinementError(val actual: TokenMatch, val expected: Token) : ErrorResult()

class SoftKeyword(name: String, private val prototype: Token) : Token(name, prototype.ignored) {
    override fun match(input: CharSequence, fromIndex: Int): Int = prototype.match(input, fromIndex)

    override fun tryParse(tokens: TokenMatchesSequence, fromPosition: Int): ParseResult<TokenMatch> =
        tryParseImpl(tokens, fromPosition)

    private tailrec fun tryParseImpl(tokens: TokenMatchesSequence, fromPosition: Int): ParseResult<TokenMatch> {
        val tokenMatch = tokens[fromPosition] ?: return UnexpectedEof(this)
        return when {
            this == tokenMatch.type -> tokenMatch.copy(type = this)
            tokenMatch.type == noneMatched -> NoMatchingToken(tokenMatch)
            tokenMatch.type.ignored -> tryParseImpl(tokens, fromPosition + 1)
            else -> MismatchedToken(this, tokenMatch)
        }
    }

    override fun equals(other: Any?): Boolean =
        other is Token && other == prototype || other is SoftKeyword && other.prototype == prototype

    override fun hashCode(): Int {
        return prototype.hashCode()
    }
}

class RefineParser(
    private val innerParser: Token,
    private val refinement: (TokenMatch) -> Boolean,
) : Parser<TokenMatch> {
    override fun tryParse(tokens: TokenMatchesSequence, fromPosition: Int) =
        when (val value = innerParser.tryParse(tokens, fromPosition)) {
            is Parsed -> if (refinement(value.value)) value
            else RefinementError(value.value, innerParser)

            is ErrorResult -> value
        }
}

fun softKeyword(value: String, prototype: Token = ExpressionToken.IDENTIFIER.token): Parser<TokenMatch> =
    RefineParser(SoftKeyword(value, prototype)) { it.text == value }
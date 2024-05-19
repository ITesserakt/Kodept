package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.map
import com.github.h0tk3y.betterParse.lexer.TokenMatch
import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.lexer.ExpressionToken.IDENTIFIER
import ru.tesserakt.kodept.lexer.ExpressionToken.RPAREN

class EndsWithParserTest : WordSpec({
    fun parser(atMost: Int = -1, atLeast: Int = 0) =
        IDENTIFIER.endsWith(RPAREN, atMost, atLeast).map { it.t1.map(TokenMatch::text) }

    "no inner tokens supplied" should {
        test(parser(), ")", emptyList())
        test(parser(), "", null)
    }

    "several inner tokens supplied" should {
        test(parser(), "a x y)", listOf("a", "x", "y"))
        test(parser(), "a x y", null)
    }

    "at least inner tokens supplied" should {
        test(parser(atLeast = 2), "a x y)", "a x y".split(" "))
        test(parser(atLeast = 2), "a x)", "a x".split(" "))
        test(parser(atLeast = 2), "a)", null)
    }

    "at most inner tokens supplied" should {
        test(parser(atMost = 2), ")", emptyList())
        test(parser(atMost = 2), "a)", listOf("a"))
        test(parser(atMost = 2), "a x)", listOf("a", "x"))
        test(parser(atMost = 2), "a x y)", null)
    }
})

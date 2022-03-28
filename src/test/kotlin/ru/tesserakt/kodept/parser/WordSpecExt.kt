package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.lexer.noneMatched
import com.github.h0tk3y.betterParse.parser.ParseException
import com.github.h0tk3y.betterParse.parser.Parser
import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import com.github.h0tk3y.betterParse.parser.tryParseToEnd
import io.kotest.assertions.throwables.shouldNotThrowAny
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.scopes.WordSpecShouldContainerScope
import io.kotest.matchers.equalityMatcher
import io.kotest.matchers.should
import io.kotest.matchers.shouldBe
import io.kotest.matchers.shouldNotHave
import ru.tesserakt.kodept.lexer.Lexer

suspend fun <T : Any> WordSpecShouldContainerScope.test(parser: Parser<T>, element: String, shouldParse: T?) =
    element.let {
        if (it.length > 20) "${it.take(20)}..." else it
    }.invoke {
        val lexer = Lexer()

        val tokens = lexer.tokenize(element)
        tokens shouldNotHave equalityMatcher(noneMatched)

        parser.tryParseToEnd(tokens, 0) should {
            when (shouldParse) {
                null -> shouldThrow<ParseException> { it.toParsedOrThrow() }
                else -> shouldNotThrowAny { it.toParsedOrThrow() }.value shouldBe shouldParse
            }
        }
    }
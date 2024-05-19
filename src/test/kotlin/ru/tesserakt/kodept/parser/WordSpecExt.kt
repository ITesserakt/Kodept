package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.combinators.map
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
import io.kotest.matchers.types.shouldBeTypeOf
import ru.tesserakt.kodept.convert
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Internal
import ru.tesserakt.kodept.core.RLT
import ru.tesserakt.kodept.core.structuralShouldBe
import ru.tesserakt.kodept.lexer.Lexer

suspend inline fun <T : Any, U : T> WordSpecShouldContainerScope.test(
    parser: Parser<T>,
    element: String,
    shouldParse: U?,
    crossinline equality: (Any, Any) -> Unit = Any::shouldBe,
) = element.let {
    if (it.length > 20) "${it.take(20)}..." else if (it.isEmpty() || it.isBlank()) "<empty string>" else it
}.invoke {
    val lexer = Lexer()

    val tokens = lexer.tokenize(element)
    tokens shouldNotHave equalityMatcher(noneMatched)

    parser.tryParseToEnd(tokens, 0) should {
        when (shouldParse) {
            null -> shouldThrow<ParseException> { it.toParsedOrThrow() }
            else -> equality(shouldNotThrowAny { it.toParsedOrThrow() }.value, shouldParse)
        }
    }
}

@OptIn(Internal::class)
suspend inline fun <T : RLT.Node, reified V : AST.Node> WordSpecShouldContainerScope.test(
    parser: Parser<T>,
    element: String,
    shouldParse: V?,
) = test(parser.map(RLT.Node::convert), element, shouldParse) { a, b ->
    a.shouldBeTypeOf<V>()
    b.shouldBeTypeOf<V>()

    a structuralShouldBe b
}
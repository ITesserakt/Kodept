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
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.convert
import ru.tesserakt.kodept.lexer.Lexer

suspend fun <T : Any, U : T> WordSpecShouldContainerScope.test(parser: Parser<T>, element: String, shouldParse: U?) =
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

suspend fun <T : RLT.Node, V : AST.Node> WordSpecShouldContainerScope.test(
    parser: Parser<T>,
    element: String,
    shouldParse: V?,
) = test(parser.map(RLT.Node::convert).map { node ->
    fun AST.Node.dfs(f: (AST.Node) -> Unit) {
        val nodeList = ArrayDeque(listOf(this))
        while (nodeList.isNotEmpty()) {
            val current = nodeList.removeFirst()
            current.children.forEach { nodeList.addFirst(it) }
            f(current)
        }
    }
    node.dfs { it.metadata.clear() }
    node
}, element, shouldParse)
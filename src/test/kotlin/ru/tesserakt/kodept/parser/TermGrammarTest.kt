package ru.tesserakt.kodept.parser

import arrow.core.NonEmptyList
import arrow.core.nonEmptyListOf
import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.lexer.toCodePoint

class TermGrammarTest : WordSpec({
    val grammar = TermGrammar

    "var ref" should {
        test(grammar.variableReference, "id", AST.UnresolvedReference("id", (1 to 1).toCodePoint()))
        test(grammar.variableReference, "123", null)
    }

    "fun ref" should {
        test(grammar.functionCall,
            "id()",
            AST.UnresolvedFunctionCall(AST.UnresolvedReference("id", (1 to 1).toCodePoint()), listOf()))
        test(
            grammar.functionCall,
            """println("Hello, world!")""",
            AST.UnresolvedFunctionCall(AST.UnresolvedReference("println", (1 to 1).toCodePoint()),
                listOf(AST.StringLiteral("Hello, world!", (1 to 9).toCodePoint())))
        )
        test(
            grammar.functionCall,
            "test((123), 10.2, foobar)",
            AST.UnresolvedFunctionCall(
                AST.UnresolvedReference("test", (1 to 1).toCodePoint()),
                listOf(
                    AST.DecimalLiteral(123.toBigInteger(), (1 to 7).toCodePoint()),
                    AST.FloatingLiteral(10.2.toBigDecimal(), (1 to 13).toCodePoint()),
                    AST.UnresolvedReference("foobar", (1 to 19).toCodePoint())
                )
            )
        )
    }

    "chain" should {
        test(
            grammar,
            "key.on()",
            AST.TermChain(
                nonEmptyListOf(
                    AST.UnresolvedReference("key", (1 to 1).toCodePoint()),
                    AST.UnresolvedFunctionCall(AST.UnresolvedReference("on", (1 to 5).toCodePoint()), listOf())
                )
            )
        )
        test(
            grammar,
            "id(x).id(x).id(x)",
            AST.TermChain(NonEmptyList.fromListUnsafe(List(3) {
                AST.UnresolvedFunctionCall(
                    AST.UnresolvedReference("id", (1 to 1 + it * 6).toCodePoint()),
                    listOf(AST.UnresolvedReference("x", (1 to 4 + it * 6).toCodePoint()))
                )
            }))
        )
        test(
            grammar, "id().id().",
            null
        )
    }
})

package ru.tesserakt.kodept.parser

import io.kotest.core.spec.style.WordSpec

class TermGrammarTest : WordSpec({
    val grammar = TermGrammar

    "var ref" should {
        test(grammar.variableReference, "id", AST.UnresolvedReference("id"))
        test(grammar.variableReference, "123", null)
    }

    "fun ref" should {
        test(grammar.functionCall, "id()", AST.UnresolvedFunctionCall(AST.UnresolvedReference("id"), listOf()))
        test(
            grammar.functionCall,
            """println("Hello, world!")""",
            AST.UnresolvedFunctionCall(AST.UnresolvedReference("println"), listOf(AST.StringLiteral("Hello, world!")))
        )
        test(
            grammar.functionCall,
            "test((123), 10.2, foobar)",
            AST.UnresolvedFunctionCall(
                AST.UnresolvedReference("test"),
                listOf(
                    AST.DecimalLiteral(123.toBigInteger()),
                    AST.FloatingLiteral(10.2.toBigDecimal()),
                    AST.UnresolvedReference("foobar")
                )
            )
        )
    }

    "chain" should {
        test(
            grammar,
            "key.on()",
            AST.TermChain(
                listOf(
                    AST.UnresolvedReference("key"),
                    AST.UnresolvedFunctionCall(AST.UnresolvedReference("on"), listOf())
                )
            )
        )
        test(
            grammar,
            "id(x).id(x).id(x)",
            AST.TermChain(List(3) {
                AST.UnresolvedFunctionCall(
                    AST.UnresolvedReference("id"), listOf(AST.UnresolvedReference("x"))
                )
            })
        )
    }
})

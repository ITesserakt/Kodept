package ru.tesserakt.kodept.parser

import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.core.AST

class TermGrammarTest : WordSpec({
    val grammar = TermGrammar

    "var ref" should {
        test(TermGrammar.variableReference, "id", AST.Reference("id"))
        test(TermGrammar.variableReference, "123", null)
    }

    "fun ref" should {
        test(
            OperatorGrammar.application, "id()", AST.FunctionCall(AST.Reference("id"), listOf())
        )
        test(
            OperatorGrammar.application, """println("Hello, world!")""", AST.FunctionCall(
                AST.Reference("println"), listOf(AST.StringLiteral("Hello, world!"))
            )
        )
        test(
            OperatorGrammar.application, "test((123), 10.2, foobar)", AST.FunctionCall(
                AST.Reference("test"), listOf(
                    AST.DecimalLiteral(123.toBigInteger()),
                    AST.FloatingLiteral(10.2.toBigDecimal()),
                    AST.Reference("foobar")
                )
            )
        )
    }

    "chain" should {
        test(
            OperatorGrammar,
            "key.on()",
            AST.FunctionCall(AST.Dereference(AST.Reference("key"), AST.Reference("on")), listOf())
        )
        test(
            OperatorGrammar, "(id(x)).(id(x)).(id(x))", AST.Dereference(
                AST.Dereference(
                    AST.FunctionCall(AST.Reference("id"), listOf(AST.Reference("x"))),
                    AST.FunctionCall(AST.Reference("id"), listOf(AST.Reference("x")))
                ), AST.FunctionCall(AST.Reference("id"), listOf(AST.Reference("x")))
            )
        )
        test(OperatorGrammar.access, "id().id().", null)
    }
})

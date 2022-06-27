package ru.tesserakt.kodept.parser

import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.core.AST

class OperatorGrammarTest : WordSpec({
    val grammar = OperatorGrammar

    "math operators" should {
        test(
            OperatorGrammar.addExpr, "123 + 321", AST.Mathematical(
                AST.DecimalLiteral(123.toBigInteger()),
                AST.DecimalLiteral(321.toBigInteger()),
                AST.Mathematical.Kind.Add,
            )
        )
        test(
            OperatorGrammar.addExpr, "123 - 321 - 0.5", AST.Mathematical(
                AST.Mathematical(
                    AST.DecimalLiteral(123.toBigInteger()),
                    AST.DecimalLiteral(321.toBigInteger()),
                    AST.Mathematical.Kind.Sub,

                    ),
                AST.FloatingLiteral(0.5.toBigDecimal()),
                AST.Mathematical.Kind.Sub,

                )
        )
        test(
            OperatorGrammar.powExpr, "a ** b ** c ** d", AST.Mathematical(
                AST.Reference("a"), AST.Mathematical(
                    AST.Reference("b"), AST.Mathematical(
                        AST.Reference("c"), AST.Reference("d"), AST.Mathematical.Kind.Pow
                    ), AST.Mathematical.Kind.Pow
                ), AST.Mathematical.Kind.Pow
            )
        )
    }

    "complex operators" should {
        test(
            OperatorGrammar, """7 & "test" + ~1.2""", AST.Binary(
                AST.DecimalLiteral(7.toBigInteger()),
                AST.Mathematical(
                    AST.StringLiteral("test"),
                    AST.BitInversion(
                        AST.FloatingLiteral(1.2.toBigDecimal()),
                    ),
                    AST.Mathematical.Kind.Add,

                    ),
                AST.Binary.Kind.And,

                )
        )
        test(
            OperatorGrammar, """2 * (2 + 2)""", AST.Mathematical(
                AST.DecimalLiteral(2.toBigInteger()),
                AST.Mathematical(
                    AST.DecimalLiteral(2.toBigInteger()),
                    AST.DecimalLiteral(2.toBigInteger()),
                    AST.Mathematical.Kind.Add,

                    ),
                AST.Mathematical.Kind.Mul,

                )
        )
        test(
            OperatorGrammar, """2 * (2 + -"αβοβα")""", AST.Mathematical(
                AST.DecimalLiteral(2.toBigInteger()),
                AST.Mathematical(
                    AST.DecimalLiteral(2.toBigInteger()),
                    AST.Negation(AST.StringLiteral("αβοβα")),
                    AST.Mathematical.Kind.Add,

                    ),
                AST.Mathematical.Kind.Mul,

                )
        )
        test(
            OperatorGrammar, """2 * 2 + 2""", AST.Mathematical(
                AST.Mathematical(
                    AST.DecimalLiteral(2.toBigInteger()),
                    AST.DecimalLiteral(2.toBigInteger()),
                    AST.Mathematical.Kind.Mul,

                    ),
                AST.DecimalLiteral(2.toBigInteger()),
                AST.Mathematical.Kind.Add,

                )
        )
        test(
            OperatorGrammar, """id(2 + 2)""", AST.FunctionCall(
                AST.Reference("id"), listOf(
                    AST.Mathematical(
                        AST.DecimalLiteral(2.toBigInteger()),
                        AST.DecimalLiteral(2.toBigInteger()),
                        AST.Mathematical.Kind.Add
                    )
                )
            )
        )
        test(
            OperatorGrammar, """core.println("Hello, " + "world!")""", AST.FunctionCall(
                AST.Dereference(
                    AST.Reference("core"), AST.Reference("println")
                ),
                listOf(
                    AST.Mathematical(
                        AST.StringLiteral("Hello, "), AST.StringLiteral("world!"), AST.Mathematical.Kind.Add
                    )
                )
            )
        )
    }
})

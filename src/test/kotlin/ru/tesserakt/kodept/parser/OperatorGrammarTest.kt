package ru.tesserakt.kodept.parser

import arrow.core.nonEmptyListOf
import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.lexer.toCodePoint

class OperatorGrammarTest : WordSpec({
    val grammar = OperatorGrammar

    "math operators" should {
        test(
            grammar.addExpr,
            "123 + 321",
            AST.Mathematical(
                AST.DecimalLiteral(123.toBigInteger(), (1 to 1).toCodePoint()),
                AST.DecimalLiteral(321.toBigInteger(), (1 to 7).toCodePoint()),
                AST.Mathematical.Kind.Add,
                (1 to 5).toCodePoint()
            )
        )
        test(
            grammar.addExpr, "123 - 321 - 0.5",
            AST.Mathematical(
                AST.Mathematical(
                    AST.DecimalLiteral(123.toBigInteger(), (1 to 1).toCodePoint()),
                    AST.DecimalLiteral(321.toBigInteger(), (1 to 7).toCodePoint()),
                    AST.Mathematical.Kind.Sub,
                    (1 to 5).toCodePoint()
                ),
                AST.FloatingLiteral(0.5.toBigDecimal(), (1 to 13).toCodePoint()),
                AST.Mathematical.Kind.Sub,
                (1 to 11).toCodePoint()
            )
        )
        test(
            grammar.elvis, "a ?: b ?: c ?: d",
            AST.Elvis(
                AST.Reference("a", (1 to 1).toCodePoint()),
                AST.Elvis(
                    AST.Reference("b", (1 to 6).toCodePoint()),
                    AST.Elvis(
                        AST.Reference("c", (1 to 11).toCodePoint()),
                        AST.Reference("d", (1 to 16).toCodePoint()),
                        (1 to 13).toCodePoint()
                    ),
                    (1 to 8).toCodePoint()
                ),
                (1 to 3).toCodePoint()
            )
        )
    }

    "complex operators" should {
        test(
            OperatorGrammar, """7 & "test" + ~1.2""",
            AST.Binary(
                AST.DecimalLiteral(7.toBigInteger(), (1 to 1).toCodePoint()),
                AST.Mathematical(
                    AST.StringLiteral("test", (1 to 5).toCodePoint()),
                    AST.BitInversion(AST.FloatingLiteral(1.2.toBigDecimal(), (1 to 15).toCodePoint()),
                        (1 to 14).toCodePoint()),
                    AST.Mathematical.Kind.Add,
                    (1 to 12).toCodePoint()
                ),
                AST.Binary.Kind.And,
                (1 to 3).toCodePoint()
            )
        )
        test(
            OperatorGrammar, """2 * (2 + 2)""",
            AST.Mathematical(
                AST.DecimalLiteral(2.toBigInteger(), (1 to 1).toCodePoint()),
                AST.Mathematical(
                    AST.DecimalLiteral(2.toBigInteger(), (1 to 6).toCodePoint()),
                    AST.DecimalLiteral(2.toBigInteger(), (1 to 10).toCodePoint()),
                    AST.Mathematical.Kind.Add,
                    (1 to 8).toCodePoint()
                ),
                AST.Mathematical.Kind.Mul,
                (1 to 3).toCodePoint()
            )
        )
        test(
            OperatorGrammar, """2 * (2 + -"αβοβα")""",
            AST.Mathematical(
                AST.DecimalLiteral(2.toBigInteger(), (1 to 1).toCodePoint()),
                AST.Mathematical(
                    AST.DecimalLiteral(2.toBigInteger(), (1 to 6).toCodePoint()),
                    AST.Negation(AST.StringLiteral("αβοβα", (1 to 11).toCodePoint()), (1 to 10).toCodePoint()),
                    AST.Mathematical.Kind.Add,
                    (1 to 8).toCodePoint()
                ),
                AST.Mathematical.Kind.Mul,
                (1 to 3).toCodePoint()
            )
        )
        test(
            OperatorGrammar, """2 * 2 + 2""",
            AST.Mathematical(
                AST.Mathematical(
                    AST.DecimalLiteral(2.toBigInteger(), (1 to 1).toCodePoint()),
                    AST.DecimalLiteral(2.toBigInteger(), (1 to 5).toCodePoint()),
                    AST.Mathematical.Kind.Mul,
                    (1 to 3).toCodePoint()
                ),
                AST.DecimalLiteral(2.toBigInteger(), (1 to 9).toCodePoint()),
                AST.Mathematical.Kind.Add,
                (1 to 7).toCodePoint()
            )
        )
        test(
            OperatorGrammar, """id(2 + 2)""",
            AST.FunctionCall(
                AST.Reference("id", (1 to 1).toCodePoint()),
                listOf(
                    AST.Mathematical(
                        AST.DecimalLiteral(2.toBigInteger(), (1 to 4).toCodePoint()),
                        AST.DecimalLiteral(2.toBigInteger(), (1 to 8).toCodePoint()),
                        AST.Mathematical.Kind.Add,
                        (1 to 6).toCodePoint()
                    )
                )
            )
        )
        test(
            OperatorGrammar, """core.println("Hello, " + "world!")""",
            AST.TermChain(
                nonEmptyListOf(
                    AST.Reference("core", (1 to 1).toCodePoint()),
                    AST.FunctionCall(
                        AST.Reference("println", (1 to 6).toCodePoint()), listOf(
                            AST.Mathematical(
                                AST.StringLiteral("Hello, ", (1 to 14).toCodePoint()),
                                AST.StringLiteral("world!", (1 to 26).toCodePoint()),
                                AST.Mathematical.Kind.Add,
                                (1 to 24).toCodePoint()
                            )
                        )
                    )
                )
            )
        )
    }
})

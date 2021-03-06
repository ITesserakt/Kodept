package ru.tesserakt.kodept.parser

import arrow.core.nonEmptyListOf
import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.core.AST

class BlockLevelGrammarTest : WordSpec({
    "function decls" should {
        test(
            BlockLevelGrammar,
            """fun println() {}""",
            AST.FunctionDecl(
                "println",
                listOf(),
                null,
                AST.ExpressionList(nonEmptyListOf(AST.TupleLiteral.unit)),
            )
        )
        test(
            BlockLevelGrammar,
            """fun println() {
                |val foo = 5
                |"term"
                |val baz = 4
            }""".trimMargin(),
            AST.FunctionDecl(
                "println", listOf(), null,
                AST.ExpressionList(
                    nonEmptyListOf(
                        AST.InitializedVar(
                            AST.Reference("foo"), false, null,
                            AST.DecimalLiteral(5.toBigInteger())
                        ),
                        AST.StringLiteral("term"),
                        AST.InitializedVar(
                            AST.Reference("baz"), false, null,
                            AST.DecimalLiteral(4.toBigInteger())
                        ),
                    ),
                ),
            )
        )
        test(
            BlockLevelGrammar, """fun println() {
                |val foo = 5
                |"term" val baz = 4
            }""".trimMargin(), null
        )
        test(
            BlockLevelGrammar,
            """fun println() {
                |val foo = 5
                |"term"; val baz = 4
            }""".trimMargin(),
            AST.FunctionDecl(
                "println", listOf(), null,
                AST.ExpressionList(
                    nonEmptyListOf(
                        AST.InitializedVar(
                            AST.Reference("foo"), false, null,
                            AST.DecimalLiteral(5.toBigInteger())
                        ),
                        AST.StringLiteral("term"),
                        AST.InitializedVar(
                            AST.Reference("baz"), false, null,
                            AST.DecimalLiteral(4.toBigInteger())
                        ),
                    ),
                ),
            )
        )
    }

    "var decls" should {
        test(BlockLevelGrammar, """var x += 5""", null)
        test(
            BlockLevelGrammar,
            """var x = 5""",
            AST.InitializedVar(AST.Reference("x"), true, null, AST.DecimalLiteral(5.toBigInteger()))
        )
        test(
            BlockLevelGrammar,
            """x += 5""",
            AST.Assignment(
                AST.Reference("x"),
                AST.Mathematical(
                    AST.Reference("x"),
                    AST.DecimalLiteral(5.toBigInteger()),
                    AST.Mathematical.Kind.Add,

                    ),

                )
        )
        test(
            BlockLevelGrammar,
            """val x = { 5 }""",
            AST.InitializedVar(
                AST.Reference("x"), false, null,
                AST.DecimalLiteral(5.toBigInteger())
            )
        )
        test(
            BlockLevelGrammar,
            """val x: Int = 3""",
            AST.InitializedVar(
                AST.Reference("x"),
                false,
                AST.Type("Int").let(AST::TypeReference),
                AST.DecimalLiteral(3.toBigInteger())
            )
        )
        test(
            BlockLevelGrammar,
            """val x: String = {}""",
            AST.InitializedVar(
                AST.Reference("x"),
                false,
                AST.Type("String").let(AST::TypeReference),
                AST.ExpressionList(nonEmptyListOf(AST.TupleLiteral.unit))
            )
        )
        test(
            BlockLevelGrammar,
            """val result = 
            |   test(blockLevelGrammar, "val x: String = {}", null)
        """.trimMargin(),
            AST.InitializedVar(
                AST.Reference("result"), false, null,
                AST.FunctionCall(
                    AST.Reference("test"),
                    listOf(
                        AST.Reference("blockLevelGrammar"),
                        AST.StringLiteral("val x: String = {}"),
                        AST.Reference("null")
                    )
                )
            )
        )
    }
})

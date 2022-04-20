package ru.tesserakt.kodept.parser

import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.lexer.toCodePoint

class BlockLevelGrammarTest : WordSpec({
    "function decls" should {
        test(
            BlockLevelGrammar, """fun println() {}""",
            AST.FunctionDecl("println",
                listOf(),
                null,
                AST.ExpressionList(listOf(), (1 to 15).toCodePoint()),
                (1 to 1).toCodePoint())
        )
        test(
            BlockLevelGrammar, """fun println() {
                |val foo = 5
                |"term"
                |val baz = 4
            }""".trimMargin(),
            AST.FunctionDecl(
                "println", listOf(), null, AST.ExpressionList(listOf(
                    AST.InitializedVar(AST.VariableDecl("foo", false, null, (2 to 1).toCodePoint()),
                        AST.DecimalLiteral(5.toBigInteger(), (2 to 11).toCodePoint())),
                    AST.StringLiteral("term", (3 to 1).toCodePoint()),
                    AST.InitializedVar(AST.VariableDecl("baz", false, null, (4 to 1).toCodePoint()),
                        AST.DecimalLiteral(4.toBigInteger(), (4 to 11).toCodePoint())),
                ), (1 to 15).toCodePoint()),
                (1 to 1).toCodePoint()
            )
        )
        test(
            BlockLevelGrammar, """fun println() {
                |val foo = 5
                |"term" val baz = 4
            }""".trimMargin(), null
        )
        test(
            BlockLevelGrammar, """fun println() {
                |val foo = 5
                |"term"; val baz = 4
            }""".trimMargin(),
            AST.FunctionDecl(
                "println", listOf(), null, AST.ExpressionList(listOf(
                    AST.InitializedVar(AST.VariableDecl("foo", false, null, (2 to 1).toCodePoint()),
                        AST.DecimalLiteral(5.toBigInteger(), (2 to 11).toCodePoint())),
                    AST.StringLiteral("term", (3 to 1).toCodePoint()),
                    AST.InitializedVar(AST.VariableDecl("baz", false, null, (3 to 9).toCodePoint()),
                        AST.DecimalLiteral(4.toBigInteger(), (3 to 19).toCodePoint())),
                ), (1 to 15).toCodePoint()),
                (1 to 1).toCodePoint()
            )
        )
    }

    "var decls" should {
        test(BlockLevelGrammar, """var x += 5""", null)
        test(
            BlockLevelGrammar,
            """var x = 5""",
            AST.InitializedVar(AST.VariableDecl("x", true, null, (1 to 1).toCodePoint()),
                AST.DecimalLiteral(5.toBigInteger(), (1 to 9).toCodePoint()))
        )
        test(
            BlockLevelGrammar, """x += 5""",
            AST.Assignment(
                AST.UnresolvedReference("x", (1 to 1).toCodePoint()),
                AST.Mathematical(
                    AST.UnresolvedReference("x", (1 to 1).toCodePoint()),
                    AST.DecimalLiteral(5.toBigInteger(), (1 to 6).toCodePoint()),
                    AST.Mathematical.Kind.Add,
                    (1 to 3).toCodePoint()
                ),
                (1 to 3).toCodePoint()
            )
        )
        test(
            BlockLevelGrammar, """val x = { 5 }""",
            AST.InitializedVar(
                AST.VariableDecl("x", false, null, (1 to 1).toCodePoint()),
                AST.ExpressionList(listOf(AST.DecimalLiteral(5.toBigInteger(), (1 to 11).toCodePoint())),
                    (1 to 9).toCodePoint())
            )
        )
        test(
            BlockLevelGrammar, """val x: Int""",
            AST.VariableDecl("x", false, AST.TypeExpression("Int", (1 to 8).toCodePoint()), (1 to 1).toCodePoint())
        )
        test(
            BlockLevelGrammar, """val x: String = {}""",
            AST.InitializedVar(AST.VariableDecl("x",
                false,
                AST.TypeExpression("String", (1 to 8).toCodePoint()),
                (1 to 1).toCodePoint()),
                AST.ExpressionList(emptyList(), (1 to 17).toCodePoint()))
        )
        test(
            BlockLevelGrammar, """val result = 
            |   test(blockLevelGrammar, "val x: String = {}", null)
        """.trimMargin(), AST.InitializedVar(
                AST.VariableDecl("result", false, null, (1 to 1).toCodePoint()),
                AST.UnresolvedFunctionCall(
                    AST.UnresolvedReference("test", (2 to 4).toCodePoint()),
                    listOf(
                        AST.UnresolvedReference("blockLevelGrammar", (2 to 9).toCodePoint()),
                        AST.StringLiteral("val x: String = {}", (2 to 28).toCodePoint()),
                        AST.UnresolvedReference("null", (2 to 50).toCodePoint())
                    )
                )
            )
        )
    }
})

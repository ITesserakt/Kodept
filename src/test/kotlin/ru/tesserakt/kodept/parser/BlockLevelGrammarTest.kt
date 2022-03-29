package ru.tesserakt.kodept.parser

import io.kotest.core.spec.style.WordSpec

class BlockLevelGrammarTest : WordSpec({
    "function decls" should {
        test(
            BlockLevelGrammar, """fun println() {}""",
            AST.FunctionDecl("println", listOf(), null, listOf())
        )
        test(
            BlockLevelGrammar, """fun println() {
                |val foo = 5
                |"term"
                |val baz = 4
            }""".trimMargin(),
            AST.FunctionDecl(
                "println", listOf(), null, listOf(
                    AST.InitializedVar(AST.VariableDecl("foo", false, null), AST.DecimalLiteral(5.toBigInteger())),
                    AST.StringLiteral("term"),
                    AST.InitializedVar(AST.VariableDecl("baz", false, null), AST.DecimalLiteral(4.toBigInteger())),
                )
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
                "println", listOf(), null, listOf(
                    AST.InitializedVar(AST.VariableDecl("foo", false, null), AST.DecimalLiteral(5.toBigInteger())),
                    AST.StringLiteral("term"),
                    AST.InitializedVar(AST.VariableDecl("baz", false, null), AST.DecimalLiteral(4.toBigInteger())),
                )
            )
        )
    }

    "var decls" should {
        test(BlockLevelGrammar, """var x += 5""", null)
        test(
            BlockLevelGrammar,
            """var x = 5""",
            AST.InitializedVar(AST.VariableDecl("x", true, null), AST.DecimalLiteral(5.toBigInteger()))
        )
        test(
            BlockLevelGrammar, """x += 5""",
            AST.Assignment(
                AST.UnresolvedReference("x"),
                AST.Mathematical(
                    AST.UnresolvedReference("x"),
                    AST.DecimalLiteral(5.toBigInteger()),
                    AST.Mathematical.Kind.Add
                )
            )
        )
        test(
            BlockLevelGrammar, """val x = { 5 }""",
            AST.InitializedVar(
                AST.VariableDecl("x", false, null),
                AST.ExpressionList(listOf(AST.DecimalLiteral(5.toBigInteger())))
            )
        )
        test(
            BlockLevelGrammar, """val x: Int""",
            AST.VariableDecl("x", false, AST.TypeExpression("Int"))
        )
        test(
            BlockLevelGrammar, """val x: String = {}""",
            null
        )
        test(
            BlockLevelGrammar, """val result = 
            |   test(blockLevelGrammar, "val x: String = {}", null)
        """.trimMargin(), AST.InitializedVar(
                AST.VariableDecl("result", false, null),
                AST.UnresolvedFunctionCall(
                    AST.UnresolvedReference("test"),
                    listOf(
                        AST.UnresolvedReference("blockLevelGrammar"),
                        AST.StringLiteral("val x: String = {}"),
                        AST.UnresolvedReference("null")
                    )
                )
            )
        )
    }
})

package ru.tesserakt.kodept.parser

import io.kotest.core.spec.style.WordSpec

class CodeFlowGrammarTest : WordSpec({
    "if expressions" should {
        test(
            CodeFlowGrammar, """if a > b { a } else { b }""",
            AST.IfExpr(
                AST.Comparison(
                    AST.UnresolvedReference("a"),
                    AST.UnresolvedReference("b"),
                    AST.Comparison.Kind.Greater
                ),
                AST.ExpressionList(listOf(AST.UnresolvedReference("a"))),
                listOf(),
                AST.IfExpr.ElseExpr(AST.ExpressionList(listOf(AST.UnresolvedReference("b"))))
            )
        )

    }

    "while expression" should {
        test(
            CodeFlowGrammar, """while s.isEmpty() { doA() }""",
            AST.WhileExpr(
                AST.TermChain(
                    listOf(
                        AST.UnresolvedReference("s"),
                        AST.UnresolvedFunctionCall(AST.UnresolvedReference("isEmpty"), listOf())
                    )
                ),
                AST.ExpressionList(listOf(AST.UnresolvedFunctionCall(AST.UnresolvedReference("doA"), listOf())))
            )
        )
    }

    "if expression in parentheses" should {
        test(
            CodeFlowGrammar, """if (x != 0) { j = 0 }""",
            AST.IfExpr(
                AST.Comparison(
                    AST.UnresolvedReference("x"),
                    AST.DecimalLiteral(0.toBigInteger()),
                    AST.Comparison.Kind.NonEqual
                ), AST.ExpressionList(
                    listOf(
                        AST.Assignment(
                            AST.UnresolvedReference("j"),
                            AST.DecimalLiteral(0.toBigInteger())
                        )
                    )
                ), listOf(), null
            )
        )
    }

    "if expr with flow operator" should {
        test(
            CodeFlowGrammar, """if i % 2 == 0 => opcode.foreignKey else => opcode.primaryKey""",
            AST.IfExpr(
                AST.Comparison(
                    AST.Mathematical(
                        AST.UnresolvedReference("i"),
                        AST.DecimalLiteral(2.toBigInteger()),
                        AST.Mathematical.Kind.Mod
                    ), AST.DecimalLiteral(0.toBigInteger()),
                    AST.Comparison.Kind.Equal
                ),
                AST.TermChain(listOf(AST.UnresolvedReference("opcode"), AST.UnresolvedReference("foreignKey"))),
                listOf(),
                AST.IfExpr.ElseExpr(
                    AST.TermChain(
                        listOf(
                            AST.UnresolvedReference("opcode"),
                            AST.UnresolvedReference("primaryKey")
                        )
                    )
                )
            )
        )

        "nested if" should {
            test(
                CodeFlowGrammar, """
                if a => b
                elif b => v
                else => if c => a 
                        elif v => c 
                        else => b
            """.trimIndent(),
                AST.IfExpr(
                    AST.UnresolvedReference("a"),
                    AST.UnresolvedReference("b"),
                    listOf(AST.IfExpr.ElifExpr(AST.UnresolvedReference("b"), AST.UnresolvedReference("v"))),
                    AST.IfExpr.ElseExpr(
                        AST.IfExpr(
                            AST.UnresolvedReference("c"),
                            AST.UnresolvedReference("a"),
                            listOf(AST.IfExpr.ElifExpr(AST.UnresolvedReference("v"), AST.UnresolvedReference("c"))),
                            AST.IfExpr.ElseExpr(AST.UnresolvedReference("b"))
                        )
                    )
                )
            )
        }
    }
})

package ru.tesserakt.kodept.parser

import arrow.core.nonEmptyListOf
import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.lexer.toCodePoint

class CodeFlowGrammarTest : WordSpec({
    "if expressions" should {
        test(
            CodeFlowGrammar, """if a > b { a } else { b }""",
            AST.IfExpr(
                AST.Comparison(
                    AST.UnresolvedReference("a", (1 to 4).toCodePoint()),
                    AST.UnresolvedReference("b", (1 to 8).toCodePoint()),
                    AST.Comparison.Kind.Greater,
                    (1 to 6).toCodePoint()
                ),
                AST.ExpressionList(listOf(AST.UnresolvedReference("a", (1 to 12).toCodePoint())),
                    (1 to 10).toCodePoint()),
                listOf(),
                AST.IfExpr.ElseExpr(AST.ExpressionList(listOf(AST.UnresolvedReference("b", (1 to 23).toCodePoint())),
                    (1 to 21).toCodePoint()), (1 to 16).toCodePoint()),
                (1 to 1).toCodePoint()
            )
        )
    }

    "while expression" should {
        test(
            CodeFlowGrammar, """while s.isEmpty() { doA() }""",
            AST.WhileExpr(
                AST.TermChain(
                    nonEmptyListOf(
                        AST.UnresolvedReference("s", (1 to 7).toCodePoint()),
                        AST.UnresolvedFunctionCall(AST.UnresolvedReference("isEmpty", (1 to 9).toCodePoint()), listOf())
                    )
                ),
                AST.ExpressionList(listOf(AST.UnresolvedFunctionCall(AST.UnresolvedReference("doA",
                    (1 to 21).toCodePoint()), listOf())), (1 to 19).toCodePoint()),
                (1 to 1).toCodePoint()
            )
        )
    }

    "if expression in parentheses" should {
        test(
            CodeFlowGrammar, """if (x != 0) { j = 0 }""",
            AST.IfExpr(
                AST.Comparison(
                    AST.UnresolvedReference("x", (1 to 5).toCodePoint()),
                    AST.DecimalLiteral(0.toBigInteger(), (1 to 10).toCodePoint()),
                    AST.Comparison.Kind.NonEqual,
                    (1 to 7).toCodePoint()
                ), AST.ExpressionList(
                    listOf(
                        AST.Assignment(
                            AST.UnresolvedReference("j", (1 to 15).toCodePoint()),
                            AST.DecimalLiteral(0.toBigInteger(), (1 to 19).toCodePoint()),
                            (1 to 17).toCodePoint()
                        )
                    ),
                    (1 to 13).toCodePoint()
                ), listOf(), null,
                (1 to 1).toCodePoint()
            )
        )
    }

    "if expr with flow operator" should {
        test(
            CodeFlowGrammar, """if i % 2 == 0 => opcode.foreignKey else => opcode.primaryKey""",
            AST.IfExpr(
                AST.Comparison(
                    AST.Mathematical(
                        AST.UnresolvedReference("i", (1 to 4).toCodePoint()),
                        AST.DecimalLiteral(2.toBigInteger(), (1 to 8).toCodePoint()),
                        AST.Mathematical.Kind.Mod,
                        (1 to 6).toCodePoint()
                    ), AST.DecimalLiteral(0.toBigInteger(), (1 to 13).toCodePoint()),
                    AST.Comparison.Kind.Equal,
                    (1 to 10).toCodePoint()
                ),
                AST.TermChain(nonEmptyListOf(AST.UnresolvedReference("opcode", (1 to 18).toCodePoint()),
                    AST.UnresolvedReference("foreignKey", (1 to 25).toCodePoint()))),
                listOf(),
                AST.IfExpr.ElseExpr(
                    AST.TermChain(
                        nonEmptyListOf(
                            AST.UnresolvedReference("opcode", (1 to 44).toCodePoint()),
                            AST.UnresolvedReference("primaryKey", (1 to 51).toCodePoint())
                        )
                    ),
                    (1 to 36).toCodePoint()
                ),
                (1 to 1).toCodePoint()
            )
        )
    }

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
                AST.UnresolvedReference("a", (1 to 4).toCodePoint()),
                AST.UnresolvedReference("b", (1 to 9).toCodePoint()),
                listOf(AST.IfExpr.ElifExpr(AST.UnresolvedReference("b", (2 to 6).toCodePoint()),
                    AST.UnresolvedReference("v", (2 to 11).toCodePoint()),
                    (2 to 1).toCodePoint())),
                AST.IfExpr.ElseExpr(
                    AST.IfExpr(
                        AST.UnresolvedReference("c", (3 to 12).toCodePoint()),
                        AST.UnresolvedReference("a", (3 to 17).toCodePoint()),
                        listOf(AST.IfExpr.ElifExpr(AST.UnresolvedReference("v", (4 to 14).toCodePoint()),
                            AST.UnresolvedReference("c", (4 to 19).toCodePoint()),
                            (4 to 9).toCodePoint())),
                        AST.IfExpr.ElseExpr(AST.UnresolvedReference("b", (5 to 17).toCodePoint()),
                            (5 to 9).toCodePoint()),
                        (3 to 9).toCodePoint()
                    ),
                    (3 to 1).toCodePoint()
                ),
                (1 to 1).toCodePoint()
            )
        )
    }
})

package ru.tesserakt.kodept.parser

import arrow.core.nonEmptyListOf
import io.kotest.core.spec.style.WordSpec
import ru.tesserakt.kodept.core.AST

class CodeFlowGrammarTest : WordSpec({
    "if expressions" should {
        test(CodeFlowGrammar,
            """if a > b { a } else { b }""",
            AST.IfExpr(
                AST.Comparison(
                    AST.Reference("a"),
                    AST.Reference("b"),
                    AST.Comparison.Kind.Greater,
                ),
                AST.Reference("a"),
                listOf(),
                AST.IfExpr.ElseExpr(
                    AST.Reference("b"),
                ),
            ))
    }

    "while expression" should {
        test(
            CodeFlowGrammar,
            """while s.isEmpty() { doA() }""",
            AST.WhileExpr(
                AST.TermChain(nonEmptyListOf(AST.Reference("s"), AST.FunctionCall(AST.Reference("isEmpty"), listOf()))),
                AST.FunctionCall(AST.Reference(
                    "doA",
                ), listOf())),
        )
    }

    "if expression in parentheses" should {
        test(CodeFlowGrammar,
            """if (x != 0) { j = 0 }""",
            AST.IfExpr(
                AST.Comparison(
                    AST.Reference("x"),
                    AST.DecimalLiteral(0.toBigInteger()),
                    AST.Comparison.Kind.NonEqual),
                AST.ExpressionList(listOf(AST.Assignment(
                    AST.Reference("j"),
                    AST.DecimalLiteral(0.toBigInteger())), AST.TupleLiteral.unit)),
                listOf(), null))
    }

    "if expr with flow operator" should {
        test(CodeFlowGrammar,
            """if i % 2 == 0 => opcode.foreignKey else => opcode.primaryKey""",
            AST.IfExpr(
                AST.Comparison(
                    AST.Mathematical(
                        AST.Reference("i"),
                        AST.DecimalLiteral(2.toBigInteger()),
                        AST.Mathematical.Kind.Mod,

                        ),
                    AST.DecimalLiteral(0.toBigInteger()),
                    AST.Comparison.Kind.Equal,

                    ),
                AST.TermChain(nonEmptyListOf(AST.Reference("opcode"), AST.Reference("foreignKey"))),
                listOf(),
                AST.IfExpr.ElseExpr(
                    AST.TermChain(nonEmptyListOf(AST.Reference("opcode"), AST.Reference("primaryKey"))),

                    ),

                ))
    }

    "nested if" should {
        test(CodeFlowGrammar,
            """
                if a => b
                elif b => v
                else => if c => a 
                        elif v => c 
                        else => b
            """.trimIndent(),
            AST.IfExpr(
                AST.Reference("a"),
                AST.Reference("b"),
                listOf(AST.IfExpr.ElifExpr(
                    AST.Reference("b"),
                    AST.Reference("v"),
                )),
                AST.IfExpr.ElseExpr(
                    AST.IfExpr(
                        AST.Reference("c"),
                        AST.Reference("a"),
                        listOf(AST.IfExpr.ElifExpr(
                            AST.Reference("v"),
                            AST.Reference("c"),
                        )),
                        AST.IfExpr.ElseExpr(
                            AST.Reference("b"),
                        ),

                        ),

                    ),

                ))
    }
})

package ru.tesserakt.kodept.visitor

import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.matchers.collections.shouldBeEmpty
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Compiler
import ru.tesserakt.kodept.core.MemoryLoader

class NodeVisitorTest : DescribeSpec({
    describe("simple visitor") {
        val compiler = Compiler(MemoryLoader.singleSnippet("""
                module test =>
                    fun a() { }
                    fun b(e: Int) { }
                    fun c() { }
                    
                    struct A
            """.trimIndent()))
        val parsed = compiler.parse().first()
        val ast = parsed.toParsedOrThrow().value

        it("should count functions in ast") {
            val visitor = object : NodeVisitor() {
                var funCount = 0

                override fun visit(node: AST.FunctionDecl) {
                    funCount++
                }
            }

            visitor.funCount shouldBe 0
            ast.root.acceptRecursively(visitor)
            visitor.funCount shouldBe 3
        }

        it("should return all constructions with property") {
            val visitor = object : NodeVisitor() {
                val fnsWithParams = mutableListOf<AST.FunctionDecl>()

                override fun visit(node: AST.FunctionDecl) {
                    if (node.params.isNotEmpty())
                        fnsWithParams += node
                }
            }

            visitor.fnsWithParams.shouldBeEmpty()
            ast.root.acceptRecursively(visitor)
            visitor.fnsWithParams shouldHaveSize 1
            visitor.fnsWithParams.first().name shouldBe "b"
        }
    }

    describe("intermediate visitor") {
        val compiler = Compiler(MemoryLoader.singleSnippet("""
                module test =>
                    fun a() { }
                    fun b(e: Int) { }
                    fun c() { }
                    
                    struct A {
                        fun test(k: Int): Int {
                            if k > 0 => k
                            else => k - 1
                        }
                    }
            """.trimIndent()))
        val parsed = compiler.parse().first()
        val ast = parsed.toParsedOrThrow().value

        it("should traverse all constructions") {
            val visitor = object : IntermediateNodeProcessor<Int>() {
                override fun visit(node: AST.Node) = 1
                override fun visit(node: AST.Leaf): Int = 0
                override fun visit(node: AST.TopLevelDecl): Int = 0
                override fun visit(node: AST.ObjectLevelDecl): Int = 0
                override fun visit(node: AST.BlockLevelDecl): Int = 0
                override fun visit(node: AST.NamedDecl): Int = 0
                override fun visit(node: AST.TypedDecl): Int = 0
                override fun visit(node: AST.CallableDecl): Int = 0
                override fun visit(node: AST.ObjectDecl): Int = 0
                override fun visit(node: AST.Expression): Int = 0
                override fun visit(node: AST.Literal): Int = 0
                override fun visit(node: AST.Operation): Int = 0
                override fun visit(node: AST.Term): Int = 0
                override fun visit(node: AST.CodeFlowExpr): Int = 0
            }

            val trueVisitor = object : NodeProcessor<Int>() {
                override fun visit(node: AST.WhileExpr): Int = 1
                override fun visit(node: AST.IfExpr): Int = 1
                override fun visit(node: AST.ExpressionList): Int = 1
                override fun visit(node: AST.CharLiteral): Int = 1
                override fun visit(node: AST.BinaryLiteral): Int = 1
                override fun visit(node: AST.DecimalLiteral): Int = 1
                override fun visit(node: AST.FloatingLiteral): Int = 1
                override fun visit(node: AST.HexLiteral): Int = 1
                override fun visit(node: AST.OctalLiteral): Int = 1
                override fun visit(node: AST.StringLiteral): Int = 1
                override fun visit(node: AST.Assignment): Int = 1
                override fun visit(node: AST.Binary): Int = 1
                override fun visit(node: AST.Comparison): Int = 1
                override fun visit(node: AST.Elvis): Int = 1
                override fun visit(node: AST.Logical): Int = 1
                override fun visit(node: AST.Mathematical): Int = 1
                override fun visit(node: AST.Absolution): Int = 1
                override fun visit(node: AST.BitInversion): Int = 1
                override fun visit(node: AST.Inversion): Int = 1
                override fun visit(node: AST.Negation): Int = 1
                override fun visit(node: AST.TermChain): Int = 1
                override fun visit(node: AST.FunctionCall): Int = 1
                override fun visit(node: AST.Reference): Int = 1
                override fun visit(node: AST.TypeExpression): Int = 1
                override fun visit(node: AST.FunctionDecl): Int = 1
                override fun visit(node: AST.FunctionDecl.Parameter): Int = 1
                override fun visit(node: AST.InitializedVar): Int = 1
                override fun visit(node: AST.VariableDecl): Int = 1
                override fun visit(node: AST.FileDecl): Int = 1
                override fun visit(node: AST.EnumDecl): Int = 1
                override fun visit(node: AST.EnumDecl.Entry): Int = 1
                override fun visit(node: AST.ModuleDecl): Int = 1
                override fun visit(node: AST.StructDecl): Int = 1
                override fun visit(node: AST.StructDecl.Parameter): Int = 1
                override fun visit(node: AST.TraitDecl): Int = 1
                override fun visit(node: AST.IfExpr.ElifExpr): Int = 1
                override fun visit(node: AST.IfExpr.ElseExpr): Int = 1
            }

            ast.root.acceptRecursively(visitor).flatten().sum() shouldBe ast.root.acceptRecursively(trueVisitor).sum()
        }
    }
})
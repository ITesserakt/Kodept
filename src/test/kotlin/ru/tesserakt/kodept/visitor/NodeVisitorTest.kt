package ru.tesserakt.kodept.visitor

import io.kotest.core.spec.style.DescribeSpec
import io.kotest.matchers.collections.shouldBeEmpty
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.MemoryLoader
import ru.tesserakt.kodept.core.shouldBeValid

class NodeVisitorTest : DescribeSpec({
    describe("simple visitor") {
        val compilationContext = CompilationContext {
            loader = MemoryLoader.singleSnippet("""
                module Test =>
                    fun a() { }
                    fun b(e: Int) { }
                    fun c() { }
                    
                    struct A
            """.trimIndent())
        }
        val ast = compilationContext flow {
            readSources().then { tokenize() }
                .then { parse() }
                .bind().shouldBeValid()
                .first()
        }

        it("should count functions in ast") {
            val visitor = object : NodeVisitor() {
                var funCount = 0

                override fun visit(node: AST.FunctionDecl) {
                    funCount++
                }
            }

            visitor.funCount shouldBe 0
            ast.value.root.acceptRecursively(visitor)
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
            ast.value.root.acceptRecursively(visitor)
            visitor.fnsWithParams shouldHaveSize 1
            visitor.fnsWithParams.first().name shouldBe "b"
        }
    }

    describe("intermediate visitor") {
        val compilationContext = CompilationContext {
            loader = MemoryLoader.singleSnippet("""
                module Test =>
                    fun a() { }
                    fun b(e: Int) { }
                    fun c() { }
                    
                    struct A {
                        fun test(k: Int): Int {
                            if k > 0 => k
                            else => k - 1
                        }
                    }
            """.trimIndent())
        }
        val ast = compilationContext flow {
            readSources().then { tokenize() }
                .then { parse() }
                .bind().shouldBeValid()
                .first()
        }

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
                override fun default(node: AST.Node) = 1
            }

            ast.value.root.acceptRecursively(visitor).flatten().sum() shouldBe ast.value.root.acceptRecursively(
                trueVisitor).sum()
        }
    }
})
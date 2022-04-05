package ru.tesserakt.kodept.visitor

import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.matchers.collections.shouldBeEmpty
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import ru.tesserakt.kodept.Compiler
import ru.tesserakt.kodept.MemoryLoader
import ru.tesserakt.kodept.parser.AST

class NodeVisitorTest : DescribeSpec({
    describe("simple visitor") {
        val compiler = Compiler {
            loader = MemoryLoader.singleSnippet(
                """
                module test =>
                    fun a() { }
                    fun b(e: Int) { }
                    fun c() { }
                    
                    struct A
            """.trimIndent()
            )
        }
        val parsed = compiler.parse().first()
        val ast = parsed.toParsedOrThrow().value

        it("should count functions in ast") {
            val visitor = object : NodeVisitor {
                var funCount = 0

                override fun visit(node: AST.FunctionDecl) {
                    funCount++
                }
            }

            visitor.funCount shouldBe 0
            ast.root.accept(visitor)
            visitor.funCount shouldBe 3
        }

        it("should return all constructions with property") {
            val visitor = object : NodeVisitor {
                val fnsWithParams = mutableListOf<AST.FunctionDecl>()

                override fun visit(node: AST.FunctionDecl) {
                    if (node.params.isNotEmpty())
                        fnsWithParams += node
                }
            }

            visitor.fnsWithParams.shouldBeEmpty()
            ast.root.accept(visitor)
            visitor.fnsWithParams shouldHaveSize 1
            visitor.fnsWithParams.first().name shouldBe "b"
        }
    }

    describe("intermediate visitor") {
        val compiler = Compiler {
            loader = MemoryLoader.singleSnippet(
                """
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
            """.trimIndent()
            )
        }
        val parsed = compiler.parse().first()
        val ast = parsed.toParsedOrThrow().value

        it("should traverse all constructions") {
            val visitor = object : IntermediateNodeVisitor {
                var nodeCount = 0

                override fun visit(node: AST.Node) {
                    println(node::class.simpleName)
                    nodeCount++
                }
            }

            visitor.nodeCount shouldBe 0
            ast.root.accept(visitor)
            visitor.nodeCount shouldBe 21
        }
    }
})
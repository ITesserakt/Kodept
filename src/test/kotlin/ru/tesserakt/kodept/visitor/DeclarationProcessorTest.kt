package ru.tesserakt.kodept.visitor

import io.kotest.core.spec.style.DescribeSpec
import io.kotest.inspectors.forAll
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.nulls.shouldBeNull
import io.kotest.matchers.nulls.shouldNotBeNull
import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.MemoryLoader
import ru.tesserakt.kodept.core.shouldBeValid

class DeclarationProcessorTest : DescribeSpec({
    describe("visitor") {
        val collector = DeclarationCollector()
        val compilationContext = CompilationContext {
            loader = MemoryLoader.singleSnippet("""
            module A {
                struct X
                struct Y
                fun x() { }
                fun y() { fun innerY() {} }
            }
            module B {
                struct X
                struct Y
                fun a() { 
                    val foo = {
                        fun print() { }
                        3
                    }
                }
                trait A { }
                enum struct Bool { True, False }
            }
        """.trimIndent())
        }

        it("should accumulate all declarations") {
            val ast = compilationContext.flow {
                readSources()
                    .then { tokenize() }
                    .then { parse() }
                    .bind().shouldBeValid()
            }.first()
            val decls = collector.collect(ast.value.root)

            decls shouldHaveSize 14

            decls[0].parent.shouldBeNull()
            decls.drop(1).take(5).forAll { it.parent.shouldNotBeNull() }
            decls.drop(6).take(1).forAll { it.parent.shouldBeNull() }
            decls.drop(7).forAll { it.parent.shouldNotBeNull() }
        }
    }
})

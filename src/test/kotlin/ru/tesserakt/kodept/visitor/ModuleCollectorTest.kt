package ru.tesserakt.kodept.visitor

import io.kotest.core.spec.style.DescribeSpec
import io.kotest.matchers.collections.shouldHaveSize
import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.MemoryLoader
import ru.tesserakt.kodept.core.shouldBeInvalid
import ru.tesserakt.kodept.core.shouldBeValid

class ModuleCollectorTest : DescribeSpec({
    describe("collector") {
        val collector = ModuleCollector()

        fun createCompiler(text: String) = CompilationContext {
            loader = MemoryLoader.singleSnippet(text)
        }

        it("should count global modules") {
            val text = """module A => struct Y"""

            val ast = createCompiler(text) flow {
                readSources().then { tokenize() }
                    .then { parse() }
                    .bind().shouldBeValid()
                    .first()
            }
            collector.collect(ast.value.root) shouldHaveSize 1
        }

        it("should count multiple modules") {
            val text = """
                module A { }
                module B { }
                module C { }
            """.trimIndent()

            val ast = createCompiler(text) flow {
                readSources().then { tokenize() }
                    .then { parse() }
                    .bind().shouldBeValid()
                    .first()
            }
            collector.collect(ast.value.root) shouldHaveSize 3
        }

        it("should not work with 0 modules") {
            val text = """"""

            createCompiler(text) flow {
                readSources().then { tokenize() }
                    .then { parse() }
                    .bind().shouldBeInvalid()
            }
        }
    }
})

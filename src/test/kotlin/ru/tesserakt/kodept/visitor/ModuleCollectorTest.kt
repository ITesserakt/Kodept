package ru.tesserakt.kodept.visitor

import io.kotest.assertions.arrow.core.shouldBeLeft
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.matchers.collections.shouldHaveSize
import ru.tesserakt.kodept.core.*

class ModuleCollectorTest : DescribeSpec({
    describe("collector") {
        val collector = ModuleCollector()

        fun createCompiler(text: String) = CompilationContext(MemoryLoader.singleSnippet(text))

        it("should count global modules") {
            val text = """module A => struct Y"""

            val ast = with(createCompiler(text)) {
                acquireContent().tokenize().parse().result
            }.map { it.value.orNull()!! }.first()
            collector.collect(ast.root) shouldHaveSize 1
        }

        it("should count multiple modules") {
            val text = """
                module A { }
                module B { }
                module C { }
            """.trimIndent()

            val ast = with(createCompiler(text)) {
                acquireContent().tokenize().parse().result
            }.map { it.value.orNull()!! }.first()
            collector.collect(ast.root) shouldHaveSize 3
        }

        it("should not work with 0 modules") {
            val text = """"""

            with(createCompiler(text)) {
                acquireContent().tokenize().parse().result
            }.first().value.shouldBeLeft()
        }
    }
})

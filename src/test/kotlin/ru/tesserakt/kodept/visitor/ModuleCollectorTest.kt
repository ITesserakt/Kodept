package ru.tesserakt.kodept.visitor

import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import io.kotest.assertions.throwables.shouldNotThrowAny
import io.kotest.assertions.throwables.shouldThrowAny
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.matchers.collections.shouldHaveSize
import ru.tesserakt.kodept.core.Compiler
import ru.tesserakt.kodept.core.MemoryLoader

class ModuleCollectorTest : DescribeSpec({
    describe("collector") {
        val collector = ModuleCollector()

        fun createCompiler(text: String) = Compiler(MemoryLoader.singleSnippet(text))

        it("should count global modules") {
            val text = """module A => struct Y"""

            val ast = shouldNotThrowAny { createCompiler(text).parse().first().toParsedOrThrow() }.value
            collector.collect(ast.root) shouldHaveSize 1
        }

        it("should count multiple modules") {
            val text = """
                module A { }
                module B { }
                module C { }
            """.trimIndent()

            val ast = shouldNotThrowAny { createCompiler(text).parse().first().toParsedOrThrow() }.value
            collector.collect(ast.root) shouldHaveSize 3
        }

        it("should not work with 0 modules") {
            val text = """"""

            shouldThrowAny { createCompiler(text).parse().first().toParsedOrThrow() }
        }
    }
})

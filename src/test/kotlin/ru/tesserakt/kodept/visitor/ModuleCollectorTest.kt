package ru.tesserakt.kodept.visitor

import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import io.kotest.assertions.throwables.shouldNotThrowAny
import io.kotest.assertions.throwables.shouldThrowAny
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.matchers.collections.shouldHaveSize
import ru.tesserakt.kodept.MemoryLoader

class ModuleCollectorTest : DescribeSpec({
    describe("collector") {
        val collector = ModuleCollector()

        fun createCompiler(text: String) = ru.tesserakt.kodept.Compiler(MemoryLoader.singleSnippet(text))

        it("should count global modules") {
            val text = """module a => struct Y"""

            val ast = shouldNotThrowAny { createCompiler(text).parse().first().toParsedOrThrow() }.value
            collector.collect(ast.root) shouldHaveSize 1
        }

        it("should count multiple modules") {
            val text = """
                module a { }
                module b { }
                module c { }
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

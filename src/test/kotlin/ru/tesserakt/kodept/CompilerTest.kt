package ru.tesserakt.kodept

import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import io.kotest.assertions.assertSoftly
import io.kotest.assertions.throwables.shouldNotThrowAny
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.matchers.sequences.shouldContainAll
import io.kotest.matchers.sequences.shouldHaveSize

class CompilerTest : DescribeSpec({
    describe("compiler") {
        it("should builds") {
            val compiler = Compiler(MemoryLoader.singleSnippet("module test =>"))

            assertSoftly(compiler) {
                acquireContents() shouldHaveSize 1
                tokenize() shouldHaveSize 1
                tokenize().first() shouldHaveSize 5
                parse() shouldHaveSize 1
            }
        }

        it("should parse files") {
            val compiler = Compiler(MemoryLoader.fromText(sequenceOf("module a => struct B", "module b =>")))

            assertSoftly(compiler) {
                acquireContents() shouldHaveSize 2
                tokenize() shouldHaveSize 2
                tokenize().map { it.count() } shouldContainAll sequenceOf(9, 5)
                parse() shouldHaveSize 2
                shouldNotThrowAny { parse().map { it.toParsedOrThrow() } }
            }
        }
    }
})

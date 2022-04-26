package ru.tesserakt.kodept.core

import io.kotest.assertions.assertSoftly
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.inspectors.forAll
import io.kotest.matchers.booleans.shouldNotBeTrue
import io.kotest.matchers.sequences.shouldContainAll
import io.kotest.matchers.sequences.shouldHaveSize

class CompilerTest : DescribeSpec({
    describe("compiler") {
        it("should builds") {
            val compilationContext = CompilationContext(MemoryLoader.singleSnippet("module test =>"))

            assertSoftly(compilationContext) {
                val source = acquireContent()
                val tokens = source.tokenize()

                source.result shouldHaveSize 1
                tokens.result shouldHaveSize 1
                tokens.result.first().value shouldHaveSize 5
                tokens.parse().result shouldHaveSize 1
            }
        }

        it("should parse files") {
            val compilationContext =
                CompilationContext(MemoryLoader.fromText(sequenceOf("module a => struct B", "module b =>")))

            assertSoftly(compilationContext) {
                val source = acquireContent()
                val tokens = source.tokenize()

                source.result shouldHaveSize 2
                tokens.result shouldHaveSize 2
                tokens.result.map { it.value.count() } shouldContainAll sequenceOf(9, 5)
                tokens.parse().result shouldHaveSize 2
                tokens.parse().result.forAll { it.value.isRight().shouldNotBeTrue() }
            }
        }
    }
})

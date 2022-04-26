package ru.tesserakt.kodept.core

import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.assertions.assertSoftly
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.inspectors.forAll
import io.kotest.matchers.sequences.shouldContainAll
import io.kotest.matchers.sequences.shouldHaveSize

class CompilerTest : DescribeSpec({
    describe("compiler") {
        it("should builds") {
            val compilationContext = CompilationContext {
                loader = MemoryLoader.singleSnippet("module Test =>")
            }

            assertSoftly(compilationContext) {
                val source = acquireContent()
                val tokens = source.tokenize()
                val parse = tokens.parse()

                source.result shouldHaveSize 1
                tokens.result shouldHaveSize 1
                tokens.result.first().value shouldHaveSize 5
                parse.result shouldHaveSize 1
                parse.result.forAll { it.value.shouldBeRight() }
            }
        }

        it("should parse files") {
            val compilationContext =
                CompilationContext {
                    loader = MemoryLoader.fromText(sequenceOf("module A => struct B", "module B =>"))
                }

            assertSoftly(compilationContext) {
                val source = acquireContent()
                val tokens = source.tokenize()
                val parse = tokens.parse()

                source.result shouldHaveSize 2
                tokens.result shouldHaveSize 2
                tokens.result.map { it.value.count() } shouldContainAll sequenceOf(9, 5)
                parse.result shouldHaveSize 2
                parse.result.forAll { it.value.shouldBeRight() }
            }
        }
    }
})

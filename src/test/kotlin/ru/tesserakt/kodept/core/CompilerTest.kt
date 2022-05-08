package ru.tesserakt.kodept.core

import io.kotest.assertions.arrow.core.shouldBeRight
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

            compilationContext flow {
                val source = readSources().bind()
                val tokens = source.tokenize().bind()
                val parse = tokens.parse().bind()

                source.source shouldHaveSize 1
                tokens.tokens shouldHaveSize 1
                tokens.tokens.first().value shouldHaveSize 5
                parse.ast shouldHaveSize 1
                parse.ast.forAll { it.value.toEither().shouldBeRight() }
            }
        }

        it("should parse files") {
            val compilationContext =
                CompilationContext {
                    loader = MemoryLoader.fromText("module A => struct B", "module B =>")
                }

            compilationContext flow {
                val source = readSources().bind()
                val tokens = source.tokenize().bind()
                val parse = tokens.parse().bind()

                source.source shouldHaveSize 2
                tokens.tokens shouldHaveSize 2
                tokens.tokens.map { it.value.count() } shouldContainAll sequenceOf(9, 5)
                parse.ast shouldHaveSize 2
                parse.ast.forAll { it.value.toEither().shouldBeRight() }
            }
        }
    }
})

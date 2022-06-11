package ru.tesserakt.kodept

import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.inspectors.forAll
import io.kotest.matchers.sequences.shouldContainAll
import io.kotest.matchers.sequences.shouldHaveSize
import ru.tesserakt.kodept.core.MemoryLoader

class CompilerTest : DescribeSpec({
    describe("compiler") {
        it("should builds") {
            val compilationContext = CompilationContext {
                loader = MemoryLoader.singleSnippet("module Test =>")
            }

            compilationContext flow {
                val source = readSources().bind()
                val tokens = source.tokenize().bind()
                val parse = tokens.parse().then { dropUnusedInfo() }.bind()

                source.source shouldHaveSize 1
                tokens.tokens shouldHaveSize 1
                tokens.tokens.first().value shouldHaveSize 4
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
                val parse = tokens.parse().then { dropUnusedInfo() }.bind()

                source.source shouldHaveSize 2
                tokens.tokens shouldHaveSize 2
                tokens.tokens.map { it.value.count() } shouldContainAll sequenceOf(8, 4)
                parse.ast shouldHaveSize 2
                parse.ast.forAll { it.value.toEither().shouldBeRight() }
            }
        }
    }
})
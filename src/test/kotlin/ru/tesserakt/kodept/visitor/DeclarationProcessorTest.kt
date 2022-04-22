package ru.tesserakt.kodept.visitor

import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import io.kotest.assertions.throwables.shouldNotThrowAny
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.inspectors.forAll
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.nulls.shouldBeNull
import io.kotest.matchers.nulls.shouldNotBeNull
import ru.tesserakt.kodept.core.Compiler
import ru.tesserakt.kodept.core.MemoryLoader

class DeclarationProcessorTest : DescribeSpec({
    describe("visitor") {
        val collector = DeclarationCollector()
        val compiler = Compiler(MemoryLoader.singleSnippet("""
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
        """.trimIndent()))

        it("should accumulate all declarations") {
            val ast = shouldNotThrowAny { compiler.parse().first().toParsedOrThrow() }.value
            val decls = collector.collect(ast.root)

            decls shouldHaveSize 14

            decls[0].parent.shouldBeNull()
            decls.drop(1).take(5).forAll { it.parent.shouldNotBeNull() }
            decls.drop(6).take(1).forAll { it.parent.shouldBeNull() }
            decls.drop(7).forAll { it.parent.shouldNotBeNull() }
        }
    }
})

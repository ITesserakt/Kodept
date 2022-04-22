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
            module a {
                struct X
                struct Y
                fun x() { }
                fun y() { fun innerY() {} }
            }
            module b {
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

            decls shouldHaveSize 12

            decls.take(4).forAll { it.parent.shouldBeNull() }
            decls[4].parent.shouldNotBeNull()
            decls.drop(5).take(3).forAll { it.parent.shouldBeNull() }
            decls.drop(8).take(2).forAll { it.parent.shouldNotBeNull() }
            decls.drop(10).take(2).forAll { it.parent.shouldBeNull() }
        }
    }
})

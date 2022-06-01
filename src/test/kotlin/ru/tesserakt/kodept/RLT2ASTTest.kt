package ru.tesserakt.kodept

import io.kotest.assertions.throwables.shouldNotThrowAny
import io.kotest.core.spec.style.FunSpec
import ru.tesserakt.kodept.core.MemoryLoader

class RLT2ASTTest : FunSpec({
    val code = """module X =>
        |   struct Y(param: Int) { 
        |       fun z(i: String) => i
        |   }
        |   
        |   trait W {
        |       abstract fun test(o: K)(m: N)
        |       
        |       fun testImpl { test(1)(2) }
        |   }
        |   
        |   enum class Bool { True, False }
        |   
        |   fun println(value: String) {
        |       val some = 1 + 2 - 5 * 4 / 0.1 % 0b1
        |       ::W::testImpl
        |       var foo
        |   }
    """.trimMargin()

    val compiler = CompilationContext {
        loader = MemoryLoader.singleSnippet(code)
    }

    context("workflow should be valid") {
        val workflow = compiler.flow {
            readSources()
                .then { tokenize() }
                .then { parse() }
                .then { abstract() }
                .bind().shouldBeValid()
        }.first()

        test("all AST nodes should have rlt initialized") {
            workflow.value.walkThrough {
                shouldNotThrowAny { it.rlt }
            }.count()
        }
    }
})
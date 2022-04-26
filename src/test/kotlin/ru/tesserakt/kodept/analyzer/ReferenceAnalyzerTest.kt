package ru.tesserakt.kodept.analyzer

import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.matchers.collections.shouldBeEmpty
import io.kotest.matchers.collections.shouldNotBeEmpty
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.transformer.ASTScopeTagger

class ReferenceAnalyzerTest : BehaviorSpec({
    given("compiler") {
        val compilationContext = ru.tesserakt.kodept.core.CompilationContext(MemoryLoader.fromText(sequenceOf(
            """module A =>
                |   fun test(a: Int) => a
            """.trimMargin(),
            """module B {
                |   enum struct Bool { True, False }
                |   
                |   fun not(self: Bool) =>
                |       if self == Bool::True => Bool::False
                |       else => Bool::True
                |}
            """.trimMargin(),
            """module C =>
                |   fun x(y: Int) { }
                |   
                |   fun z() => y
            """.trimMargin()
        ))) {
            transformers = listOf(::ASTScopeTagger)
        }

        `when`("text parsed") {
            val ast = with(compilationContext) {
                acquireContent().tokenize().parse().transform().result
            }.toList()

            and("analyzed with Reference analyzer") {
                val analyzer = ReferenceAnalyzer()

                then("the first entry should not produce any reports") {
                    analyzer.analyzeIndependently(ast[0].value.orNull()!!)
                    analyzer.collectedReports.shouldBeEmpty()
                }

                then("the second entry should not produce any reports") {
                    analyzer.analyzeIndependently(ast[1].value.orNull()!!)
                    analyzer.collectedReports.shouldBeEmpty()
                }

                then("the third entry should produce reports") {
                    analyzer.analyzeIndependently(ast[2].value.orNull()!!)
                    analyzer.collectedReports.shouldNotBeEmpty()
                }
            }
        }
    }
})

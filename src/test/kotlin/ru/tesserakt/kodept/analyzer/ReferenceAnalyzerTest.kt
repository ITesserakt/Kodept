package ru.tesserakt.kodept.analyzer

import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.matchers.collections.shouldBeEmpty
import io.kotest.matchers.collections.shouldNotBeEmpty
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.UnrecoverableError
import ru.tesserakt.kodept.transformer.ASTScopeTagger

class ReferenceAnalyzerTest : BehaviorSpec({
    given("compiler") {
        val compilationContext = CompilationContext {
            loader = MemoryLoader.fromText(
                """module A =>
                |   fun test(a: Int) => a
            """.trimMargin(),
                """module C =>
                |   fun x(y: Int) { }
                |   
                |   fun z() => y
            """.trimMargin())
            transformers = listOf(::ASTScopeTagger)
        }

        `when`("text parsed") {
            val ast = with(compilationContext) {
                acquireContent().tokenize().parse().transform().result
            }.map { it.value.toEither().shouldBeRight() }.toList()

            and("analyzed with Reference analyzer") {
                val analyzer = ReferenceAnalyzer()

                then("the first entry should not produce any reports") {
                    try {
                        analyzer.analyzeIndependently(ast[0])
                    } catch (_: UnrecoverableError) {
                    }
                    analyzer.collectedReports.shouldBeEmpty()
                }

                then("the third entry should produce reports") {
                    try {
                        analyzer.analyzeIndependently(ast[1])
                    } catch (_: UnrecoverableError) {
                    }
                    analyzer.collectedReports.shouldNotBeEmpty()
                }
            }
        }
    }
})

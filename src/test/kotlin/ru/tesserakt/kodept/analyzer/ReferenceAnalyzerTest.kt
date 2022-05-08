package ru.tesserakt.kodept.analyzer

import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.matchers.collections.shouldBeEmpty
import io.kotest.matchers.collections.shouldNotBeEmpty
import ru.tesserakt.kodept.core.CompilationContext
import ru.tesserakt.kodept.core.MemoryLoader
import ru.tesserakt.kodept.core.shouldBeValid
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
            val ast = compilationContext flow {
                readSources().then { tokenize() }
                    .then { parse() }
                    .then { applyTransformations() }
                    .then { analyze() }
                    .bind().shouldBeValid()
            }

            and("analyzed with Reference analyzer") {
                val analyzer = ReferenceAnalyzer()

                then("the first entry should not produce any reports") {
                    try {
                        analyzer.analyzeIndependently(ast[0].value)
                    } catch (_: UnrecoverableError) {
                    }
                    analyzer.collectedReports.shouldBeEmpty()
                }

                then("the third entry should produce reports") {
                    try {
                        analyzer.analyzeIndependently(ast[1].value)
                    } catch (_: UnrecoverableError) {
                    }
                    analyzer.collectedReports.shouldNotBeEmpty()
                }
            }
        }
    }
})

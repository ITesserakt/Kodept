package ru.tesserakt.kodept.analyzer

import io.kotest.core.spec.style.DescribeSpec
import io.kotest.inspectors.forAll
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import ru.tesserakt.kodept.core.*
import ru.tesserakt.kodept.error.Report

class ModuleAnalyzerTest : DescribeSpec({
    describe("compiler") {
        val compilationContext = CompilationContext {
            loader = MemoryLoader.fromText(sequenceOf(
                """module A =>""",
                """module A {  }
                |module B {  }
            """.trimMargin(),
                """module A {  }
                |module A {  }
                |module B {  }
                |module C {  }
                |module B {  }
            """.trimMargin()
            ))
            analyzers = listOf(ModuleAnalyzer())
        }

        describe("it flow") {
            val flow = with(compilationContext) {
                acquireContent().tokenize().parse().transform().analyze().result
            }
            val reports = flow.mapWithFilename {
                it.fold({ it.toList() }, { emptyList() }, { a, b -> a.toList() })
            }.map { (list, file) ->
                list.map { FileRelative(it, file) }
            }.flatten().toList()

            it("analyzer should produce right reports") {
                reports shouldHaveSize 2
                reports.forAll {
                    it.value.file shouldBe it.filename
                    it.value.severity shouldBe Report.Severity.ERROR
                }
            }
        }
    }
})

package ru.tesserakt.kodept.analyzer

import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.inspectors.forAll
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import ru.tesserakt.kodept.core.Compiler
import ru.tesserakt.kodept.core.MemoryLoader
import ru.tesserakt.kodept.error.Report

class ModuleAnalyzerTest : DescribeSpec({
    describe("compiler") {
        val compiler = Compiler(MemoryLoader.fromText(sequenceOf(
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
        )))

        it("analyzer should produce right reports") {
            val analyzer = ModuleAnalyzer()
            val reports = analyzer.analyze(compiler.parse().map { it.toParsedOrThrow().value }).value()

            reports shouldHaveSize 2
            reports.forAll {
                it.file shouldBe compiler.acquireContents().toList()[2].name
                it.severity shouldBe Report.Severity.ERROR
            }
        }
    }
})

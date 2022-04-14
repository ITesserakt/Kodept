package ru.tesserakt.kodept.analyzer

import com.github.h0tk3y.betterParse.parser.toParsedOrThrow
import io.kotest.core.spec.style.DescribeSpec
import io.kotest.inspectors.forAll
import io.kotest.matchers.collections.shouldHaveSize
import io.kotest.matchers.shouldBe
import ru.tesserakt.kodept.MemoryLoader
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.SemanticError

class ModuleAnalyzerTest : DescribeSpec({
    describe("compiler") {
        val compiler = ru.tesserakt.kodept.Compiler(MemoryLoader.fromText(sequenceOf(
            """module a =>""",
            """module a {  }
                |module b {  }
            """.trimMargin(),
            """module a {  }
                |module a {  }
                |module b {  }
                |module c {  }
                |module b {  }
            """.trimMargin()
        )))

        it("analyzer should produce right reports") {
            val analyzer = ModuleAnalyzer()
            val reports = analyzer.analyze(compiler.parse().map { it.toParsedOrThrow().value }).toList()

            reports shouldHaveSize 2
            reports.forAll {
                it.file shouldBe compiler.acquireContents().toList()[2].name
                it.severity shouldBe Report.Severity.ERROR
            }
            reports.first().message shouldBe SemanticError.DuplicatedModules("a")
            reports.last().message shouldBe SemanticError.DuplicatedModules("b")
        }
    }
})

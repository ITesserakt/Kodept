package ru.tesserakt.kodept.cli

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.requireObject
import com.github.ajalt.clikt.parameters.groups.provideDelegate
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import mu.KotlinLogging
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.ProgramCodeHolder
import ru.tesserakt.kodept.core.asString
import ru.tesserakt.kodept.flowable.ParsedContent
import using

private val logger = KotlinLogging.logger("[Compiler]")

class Typecheck : CliktCommand(help = "- typecheck programs and show inferred types of functions") {
    private val parsed by requireObject<Triple<CompilationContext, ParsedContent.Data, ProgramCodeHolder>>()
    private val reportOptions by ReportProcessorOptions()
    private val printTree by option("--tree", "-t", help = "Print parsed tree with transformations").flag()

    override fun run() {
        val (context, parsed, code) = parsed
        val (result) = context workflow {
            parsed.analyze().bind()
        }

        with(logger) {
            using(reportOptions.processor, code) {
                result.forEach { (it, _) ->
                    it.printReportsOr {
                        if (printTree) it.asString()
                        else ""
                    }
                }
            }
        }
    }
}
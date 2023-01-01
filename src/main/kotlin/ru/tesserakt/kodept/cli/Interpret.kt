package ru.tesserakt.kodept.cli

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.requireObject
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.multiple
import com.github.ajalt.clikt.parameters.groups.provideDelegate
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import mu.KotlinLogging
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.ProgramCodeHolder
import ru.tesserakt.kodept.flowable.ParsedContent
import using

private val logger = KotlinLogging.logger("[Compiler]")

class Interpret : CliktCommand(help = "- run programs using interpreter") {
    private val parsed by requireObject<Triple<CompilationContext, ParsedContent.Data, ProgramCodeHolder>>()
    private val reportOptions by ReportProcessorOptions()
    private val computeLazy by option("--lazy", "-l", help = "Defer computation").flag()
    private val arguments by argument().multiple()

    override fun run() {
        val (context, parsed, code) = parsed
        val (output) = context workflow {
            parsed.analyze()
                .then { interpret(lazily = computeLazy, args = arguments) }
                .bind()
        }

        with(reportOptions.processor) {
            using(code, logger) {
                output.value().printReportsOr { (_, out) ->
                    "exited with: $out"
                }
            }
        }
    }
}
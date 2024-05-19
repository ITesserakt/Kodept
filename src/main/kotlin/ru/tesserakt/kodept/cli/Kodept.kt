package ru.tesserakt.kodept.cli

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.choice
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.Loader
import ru.tesserakt.kodept.defaultAnalyzers
import ru.tesserakt.kodept.defaultTransformers

class Kodept : CliktCommand(
    printHelpOnEmptyArgs = true, help = """
        |Typechecks or interprets passed FILE or DIR using Kodept programming language
        |
        |Example: ./kodept parse /absolute/path/ <typecheck | interpret>
        |
        |See `parse` help for more info
    """.trimMargin(),
    epilog = """Bauman Moscow State Technical University - BMSTU - 2022"""
) {
    private val debug by option("-d", "--debug", help = "Enable debugging output").flag()
    private val verbose by option("-v", "--verbose", help = "Enable verbose output").flag()
    private val verbosity by option(
        "-s", "--severity", help = "Set logger verbosity", envvar = "org.slf4j.simpleLogger.defaultLogLevel"
    ).choice("trace", "debug", "info", "warn", "error")

    override fun run() {
        val mode = if (debug) "debug"
        else if (verbose) "trace"
        else verbosity ?: "info"

        System.setProperty("org.slf4j.simpleLogger.defaultLogLevel", mode)

        fun context(loader: Loader) = CompilationContext {
            this.loader = loader
            transformers = defaultTransformers
            analyzers = defaultAnalyzers
        }

        currentContext.obj = ::context
    }
}
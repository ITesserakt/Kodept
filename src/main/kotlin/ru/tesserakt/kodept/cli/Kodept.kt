package ru.tesserakt.kodept.cli

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.validate
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.choice
import com.github.ajalt.clikt.parameters.types.path
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.defaultAnalyzers
import ru.tesserakt.kodept.defaultTransformers

class Kodept :
    CliktCommand(printHelpOnEmptyArgs = true, help = "- process passed programs (see `parse` help for more info)") {
    private val debug by option(help = "Enable debugging output").flag()

    private val verbosity by option(
        help = "Set logger verbosity",
        envvar = "org.slf4j.simpleLogger.defaultLogLevel"
    ).choice("trace", "debug", "info", "warn", "error")

    private val file by argument(help = "File or directory to process").path(mustExist = true).validate {
        require(it.isAbsolute) { "Provided path should be absolute" }
    }

    private val extension by option(help = "File extension to work with")
    private val anyExtension by option(help = "Accept all files in the given path").flag()

    override fun run() {
        val mode = if (debug) "debug"
        else verbosity ?: "info"

        System.setProperty("org.slf4j.simpleLogger.defaultLogLevel", mode)

        val context = CompilationContext {
            loader = FileLoader {
                path = file
                anySourceExtension = anyExtension
                this@Kodept.extension?.let { extension = it }
            }
            transformers = defaultTransformers
            analyzers = defaultAnalyzers
        }

        currentContext.obj = context
    }
}
package ru.tesserakt.kodept.cli

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.requireObject
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import ru.tesserakt.kodept.CompilationContext

class Parse : CliktCommand(help = "- parse files and do operations (see available commands)") {
    val allErrors by option(help = "Show all errors while parsing").flag("less-errors", defaultForHelp = "less-errors")
    val context by requireObject<CompilationContext>()

    override fun run() {
        val result = context workflow {
            val sources = readSources()
            sources
                .then { tokenize() }
                .then { parse(!allErrors) }
                .then { dropUnusedInfo() }
                .also { sources.bind().holder }
        }

        currentContext.findOrSetObject { Triple(context, result.first, result.second) }
    }
}
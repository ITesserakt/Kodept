package ru.tesserakt.kodept.cli

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.requireObject
import com.github.ajalt.clikt.parameters.groups.OptionGroup
import com.github.ajalt.clikt.parameters.groups.groupChoice
import com.github.ajalt.clikt.parameters.groups.provideDelegate
import com.github.ajalt.clikt.parameters.groups.required
import com.github.ajalt.clikt.parameters.options.*
import com.github.ajalt.clikt.parameters.types.path
import io.github.oshai.kotlinlogging.KotlinLogging
import ru.tesserakt.kodept.CompilationContext
import ru.tesserakt.kodept.core.FileLoader
import ru.tesserakt.kodept.core.Loader
import ru.tesserakt.kodept.core.MemoryLoader
import using

private val logger = KotlinLogging.logger("[Compiler]")

sealed class LoadConfig(name: String) : OptionGroup(name) {
    abstract val loader: Loader
}

class FileConfig : LoadConfig("Options for loading programs from file or directory") {
    private val file by option(help = "File or directory to process").path(mustExist = true).required().validate {
        require(it.isAbsolute) { "Provided path should be absolute" }
    }
    private val ext by option("-e", "--extension", help = "File extension to work with").default("kd")
    private val anyExt by option("-a", "--any-extension", help = "Accept all files in the given path").flag()

    override val loader by lazy {
        FileLoader {
            path = file
            extension = ext
            anySourceExtension = anyExt
        }
    }
}

class MemoryConfig : LoadConfig("Options for loading from console") {
    override val loader by lazy {
        val text = generateSequence { readlnOrNull() }.joinToString("\n")
        MemoryLoader.singleSnippet(text)
    }
}

class Parse : CliktCommand(
    help = "- parse files and do operations (see available commands)", invokeWithoutSubcommand = true
) {
    private val allErrors by option("--all", help = "Show all errors while parsing").flag(
        "--less", defaultForHelp = "--less"
    )
    private val loadConfig by option(help = "Config to load programs").groupChoice(
        "file" to FileConfig(), "console" to MemoryConfig()
    ).required()
    private val contextFn by requireObject<(Loader) -> CompilationContext>()
    private val context by lazy { contextFn(loadConfig.loader) }
    private val reportOptions by ReportProcessorOptions()

    override fun run() {
        val subcommand = currentContext.invokedSubcommand

        val result = context workflow {
            val sources = readSources()
            sources.then { tokenize() }.then { parse(!allErrors) }.then { dropUnusedInfo() }
                .also { sources.bind().holder }
        }

        if (subcommand != null) currentContext.findOrSetObject { Triple(context, result.first, result.second) }
        else using(reportOptions.processor, result.second, logger) {
            result.first.ast.forEach { it.value.printReportsOr { "" } }
        }
    }
}
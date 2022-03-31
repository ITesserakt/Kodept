package ru.tesserakt.kodept

import java.io.File
import java.io.InputStream
import java.nio.file.Path
import kotlin.io.path.Path
import kotlin.io.path.absolute
import kotlin.io.path.isDirectory

interface Loader {
    fun getSources(): Sequence<CodeSource>

    fun loadSources() = getSources().map { it.getContents() }
}

class FileLoader private constructor(private val files: () -> Sequence<File>) : Loader {
    override fun getSources(): Sequence<CodeSource> = files().map { FileCodeSource(it) }

    class BuilderScope {
        var path: Path = Path("").absolute()
        var extension = "kd"
        var anyExtension = false

        fun build() = require(path.isDirectory()) { "Provided path should be directory" }.let {
            path.toFile()
                .walkTopDown()
                .filter { it.isFile }
                .filter { anyExtension || it.extension == extension }
                .let { FileLoader { it } }
        }
    }

    companion object {
        inline operator fun invoke(scope: BuilderScope.() -> Unit = {}) = BuilderScope().apply(scope).build()
    }
}

class MemoryLoader private constructor(private val streams: Sequence<InputStream>) : Loader {
    override fun getSources(): Sequence<CodeSource> = streams.map(::MemoryCodeSource)

    companion object {
        fun fromText(text: Sequence<String>) = MemoryLoader(text.map { it.byteInputStream() })

        fun singleSnippet(text: String) = fromText(sequenceOf(text))
    }
}
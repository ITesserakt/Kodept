package ru.tesserakt.kodept.core

import java.io.File
import java.nio.file.Path
import java.time.Instant
import kotlin.io.path.Path
import kotlin.io.path.absolute
import kotlin.io.path.extension
import kotlin.io.path.isDirectory

interface Loader {
    fun getSources(): Sequence<CodeSource>

    fun loadSources() = getSources().map { it.contents }
}

class FileLoader private constructor(private val files: Sequence<File>, caches: Sequence<File>) : Loader {
    override fun getSources(): Sequence<CodeSource> = files.map(::FileCodeSource)

    class BuilderScope {
        var path: Path = Path("").absolute()
        var extension = "kd"
        var anySourceExtension = false
        var cacheExtension = "kdc"

        private var cacheFolder: Path? = null

        fun withCaches(scope: Path) {
            cacheFolder = scope
        }

        fun build(): FileLoader {
            require(cacheFolder?.absolute()?.isDirectory() ?: true) { "Provided cache path should be directory" }
            val caches = cacheFolder?.toFile()
                ?.walkTopDown().orEmpty()
                .filter { it.isFile }
                .filter { it.extension == cacheExtension }

            val files = if (path.isDirectory()) {
                require(path.isAbsolute) { "Provided path should be absolute" }
                path.toFile()
                    .walkTopDown()
                    .filter { it.isFile }
                    .filter { anySourceExtension || it.extension == extension }
            } else if (path.toFile().isFile && (anySourceExtension || path.extension == extension)) {
                sequenceOf(path.toFile())
            } else {
                emptySequence()
            }

            return FileLoader(files, caches)
        }
    }

    companion object {
        inline operator fun invoke(scope: BuilderScope.() -> Unit = {}) = BuilderScope().apply(scope).build()
    }
}

class MemoryLoader private constructor(private val streams: Iterable<String>) : Loader {
    // workaround to eager time computation
    override fun getSources(): Sequence<CodeSource> = streams.mapIndexed { i, it ->
        "scratch-#$i-${Instant.now().epochSecond}.kd" to it
    }.asSequence().map { (name, text) ->
        MemoryCodeSource(text.byteInputStream(), name)
    }

    companion object {
        fun fromText(text: Iterable<String>) = MemoryLoader(text)

        fun fromText(vararg text: String) = MemoryLoader(text.toList())

        fun singleSnippet(text: String) = fromText(arrayListOf(text))
    }
}

class PreCachedLoader(private val caches: Sequence<CacheData>) : Loader {
    override fun getSources(): Sequence<CodeSource> = caches.map(::CachedCodeSource)
}
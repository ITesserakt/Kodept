package ru.tesserakt.kodept.core

import com.google.gson.Gson
import java.io.File
import java.io.InputStream
import java.nio.file.Path
import java.time.Instant
import kotlin.io.path.Path
import kotlin.io.path.absolute
import kotlin.io.path.isDirectory

interface Loader {
    fun getSources(): Sequence<CodeSource>

    fun loadSources() = getSources().map { it.contents }
}

class FileLoader private constructor(private val files: Sequence<File>, caches: Sequence<File>) : Loader {
    private val parsed = caches.map {
        Gson().fromJson(it.readText(), CacheData::class.java)
    }

    override fun getSources(): Sequence<CodeSource> = files.map(::FileCodeSource) + parsed.map(::CachedCodeSource)

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

            require(path.isDirectory()) { "Provided path should be directory" }
            val files = path.toFile()
                .walkTopDown()
                .filter { it.isFile }
                .filter { anySourceExtension || it.extension == extension }

            return FileLoader(files, caches)
        }
    }

    companion object {
        inline operator fun invoke(scope: BuilderScope.() -> Unit = {}) = BuilderScope().apply(scope).build()
    }
}

class MemoryLoader private constructor(private val streams: Sequence<InputStream>) : Loader {
    override fun getSources(): Sequence<CodeSource> = streams.mapIndexed { i, it ->
        MemoryCodeSource(it, "scratch-#$i-${Instant.now().epochSecond}.kd")
    }

    override fun loadSources(): Sequence<InputStream> = streams

    companion object {
        fun fromText(text: Sequence<String>) = MemoryLoader(text.map { it.byteInputStream() })

        fun singleSnippet(text: String) = fromText(sequenceOf(text))
    }
}

class PreCachedLoader(private val caches: Sequence<CacheData>) : Loader {
    override fun getSources(): Sequence<CodeSource> = caches.map(::CachedCodeSource)
}
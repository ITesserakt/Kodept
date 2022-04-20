package ru.tesserakt.kodept.core

import java.io.File
import java.io.InputStream

interface CodeSource {
    val contents: InputStream
    val hint: AST?
    val name: String
}

@JvmInline
value class FileCodeSource(private val file: File) : CodeSource {
    override val contents get() = file.inputStream()
    override val hint: AST? get() = null
    override val name: String get() = file.absolutePath
}

class MemoryCodeSource(private val stream: InputStream, override val name: String) : CodeSource {
    override val contents: InputStream get() = stream
    override val hint: AST? get() = null
}

@JvmInline
value class CachedCodeSource(private val data: CacheData) : CodeSource {
    override val contents: InputStream get() = data.source.byteInputStream()
    override val hint: AST get() = data.ast
    override val name: String get() = data.sourceName
}
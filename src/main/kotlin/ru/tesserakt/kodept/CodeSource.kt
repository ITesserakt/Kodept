package ru.tesserakt.kodept

import java.io.File
import java.io.InputStream

interface CodeSource {
    fun getContents(): InputStream

    val name: String
}

@JvmInline
value class FileCodeSource(private val file: File) : CodeSource {
    override fun getContents() = file.inputStream()

    override val name: String get() = file.name
}

class MemoryCodeSource(private val stream: InputStream, override val name: String) : CodeSource {
    override fun getContents(): InputStream = stream
}
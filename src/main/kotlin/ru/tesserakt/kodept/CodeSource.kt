package ru.tesserakt.kodept

import java.io.File
import java.io.InputStream

interface CodeSource {
    fun getContents(): InputStream
}

@JvmInline
value class FileCodeSource(private val file: File) : CodeSource {
    override fun getContents() = file.inputStream()
}

@JvmInline
value class MemoryCodeSource(private val stream: InputStream) : CodeSource {
    override fun getContents(): InputStream = stream
}
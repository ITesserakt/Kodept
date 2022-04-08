package ru.tesserakt.kodept

import java.io.ByteArrayOutputStream
import java.io.File
import java.io.OutputStream

interface Cache {
    val stream: OutputStream
}

class FileCache(file: File) : Cache {
    override val stream: OutputStream = file.outputStream().buffered()
}

class MemoryCache : Cache {
    override val stream = ByteArrayOutputStream()

    fun getOutput() = stream.toString()
}

fun Cache.getOutput() = when (this) {
    is MemoryCache -> getOutput()
    else -> null
}
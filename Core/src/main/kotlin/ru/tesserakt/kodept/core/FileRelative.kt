package ru.tesserakt.kodept.core

import java.io.File

@JvmInline
value class Filepath(val name: String) {
    val asFile get() = File(name)
    val extension get() = asFile.extension
}

data class FileRelative<out T>(val value: T, val filepath: Filepath)

inline fun <T> CodeSource.withFilename(f: CodeSource.() -> T) = FileRelative(this.f(), Filepath(name))

inline fun <T, V> FileRelative<T>.map(f: (T) -> V) = FileRelative(f(value), filepath)

inline fun <T, U> Sequence<FileRelative<T>>.mapWithFilename(crossinline f: Filepath.(T) -> U) =
    map { rel -> rel.map { rel.filepath.f(it) } }
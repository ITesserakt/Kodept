package ru.tesserakt.kodept.core

typealias Filename = String

data class FileRelative<out T>(val value: T, val filename: Filename)

inline fun <T> CodeSource.withFilename(f: CodeSource.() -> T) = FileRelative(this.f(), name)

inline fun <T, V> FileRelative<T>.map(f: (T) -> V) = FileRelative(f(value), filename)

inline fun <T, U> Sequence<FileRelative<T>>.mapWithFilename(crossinline f: Filename.(T) -> U) =
    map { rel -> rel.map { rel.filename.f(it) } }
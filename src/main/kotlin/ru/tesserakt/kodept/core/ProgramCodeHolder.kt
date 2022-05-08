package ru.tesserakt.kodept.core

import arrow.core.Eval

class ProgramCodeHolder(private val text: Map<Filename, Eval<String>>) {
    operator fun get(filename: Filename) = Accessor(filename)
    fun <T> walkThoughAll(f: (Accessor) -> T) = text.map { Accessor(it.key) }.asSequence().map(f)

    inner class Accessor(accessToken: Filename) {
        private val cached = text[accessToken] ?: throw IllegalArgumentException("Unknown file passed: $accessToken")
        private val lines = cached.map { it.lines() }

        val filename = accessToken

        val allText get() = cached.value()

        fun range(range: IntRange) = allText.substring(range)

        fun specificLine(lineIdx: Int) = lines.value().elementAt(lineIdx)

        fun linesRange(range: IntRange) = lines.value().subList(range.first, range.last + 1)
    }

    override fun toString() = "ProgramCodeHolder(size = ${text.size})"
}
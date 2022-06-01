package ru.tesserakt.kodept.core

import com.github.h0tk3y.betterParse.lexer.TokenMatch

data class CodePoint(val line: Int, val position: Int, val length: Int = 1) {
    override fun toString(): String = "$line:$position"
}

fun TokenMatch.toCodePoint() = CodePoint(row, column, length)
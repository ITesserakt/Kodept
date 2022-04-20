package ru.tesserakt.kodept.lexer

import com.github.h0tk3y.betterParse.lexer.TokenMatch

data class CodePoint(val line: Int, val position: Int) {
    override fun toString(): String = "$line:$position"
}

fun TokenMatch.toCodePoint() = CodePoint(row, column)
fun Pair<Int, Int>.toCodePoint() = CodePoint(first, second)
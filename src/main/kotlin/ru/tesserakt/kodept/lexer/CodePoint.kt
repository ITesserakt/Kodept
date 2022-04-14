package ru.tesserakt.kodept.lexer

import com.github.h0tk3y.betterParse.lexer.TokenMatch

data class CodePoint(val line: Int, val position: Int)

fun TokenMatch.toCodePoint() = CodePoint(row, column)
fun Pair<Int, Int>.toCodePoint() = CodePoint(first, second)
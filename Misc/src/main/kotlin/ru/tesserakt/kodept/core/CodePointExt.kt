package ru.tesserakt.kodept.core

import com.github.h0tk3y.betterParse.lexer.TokenMatch

fun TokenMatch.toCodePoint() = CodePoint(row, column, length)
package ru.tesserakt.kodept.parser

import com.github.h0tk3y.betterParse.parser.Parser

fun interface Rule<T> {
    operator fun invoke(): Parser<T>
}
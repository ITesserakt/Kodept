package ru.tesserakt.kodept.core

import com.github.h0tk3y.betterParse.parser.ErrorResult

fun interface ParseHandle<T> {
    fun onError(file: String, error: ErrorResult): T
}
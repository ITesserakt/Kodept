package ru.tesserakt.kodept.core

data class CodePoint(val line: Int, val position: Int, val length: Int = 1) {
    override fun toString(): String = "$line:$position"
}
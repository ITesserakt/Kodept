package ru.tesserakt.kodept.analyzer

import ru.tesserakt.kodept.AST
import ru.tesserakt.kodept.error.Report

fun interface Analyzer {
    fun analyze(files: Sequence<AST>): Sequence<Report>
}
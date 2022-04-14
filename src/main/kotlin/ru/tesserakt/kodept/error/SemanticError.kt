package ru.tesserakt.kodept.error

sealed class SemanticError(val code: String, val message: String) {
    data class DuplicatedModules(val duplicate: String) :
        SemanticError("K1", "File contains duplicated module $duplicate")
}
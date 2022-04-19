package ru.tesserakt.kodept.error

sealed class SemanticError(val code: String, val message: String) {
    data class DuplicatedModules(val duplicate: String) :
        SemanticError("K1", "File contains duplicated module $duplicate")

    data class DuplicatedVariables(val module: String, val duplicate: String) :
        SemanticError("K2", "Module $module contains duplicated variable $duplicate")
}
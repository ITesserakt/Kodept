package ru.tesserakt.kodept.error

import arrow.core.NonEmptyList
import ru.tesserakt.kodept.lexer.CodePoint

sealed class SemanticError(override val code: String, override val message: String) : ReportMessage {
    data class DuplicatedModules(val duplicates: NonEmptyList<Pair<CodePoint, String>>) :
        SemanticError(
            "KE1",
            "File contains duplicated module ${duplicates.head.second} at ${duplicates.map { it.first }}"
        )

    data class UndeclaredUsage(val name: String) :
        SemanticError("KE3", "Usage of undeclared $name")

    data class AmbitiousReference(val name: String) :
        SemanticError("KE4", "Ambitious usage of $name")
}
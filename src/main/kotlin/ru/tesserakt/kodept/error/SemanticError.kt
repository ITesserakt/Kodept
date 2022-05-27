package ru.tesserakt.kodept.error

sealed class SemanticError(final override val code: String, override val message: String) : ReportMessage {
    init {
        require(code.startsWith("KSeE"))
    }

    data class DuplicatedModules(val moduleName: String) :
        SemanticError(
            "KSeE1",
            "File contains duplicated module `$moduleName`"
        )

    data class UndeclaredUsage(val name: String) :
        SemanticError("KSeE3", "Usage of undeclared $name")

    data class AmbitiousReference(val name: String) :
        SemanticError("KSeE4", "Ambitious usage of $name")

    data class DuplicatedVariable(val name: String) :
        SemanticError("KSeE5", "Variable `$name` has duplicates in one block")

    object Duplicated : SemanticError("KSeE6", "Declaration has duplicates across block")
}
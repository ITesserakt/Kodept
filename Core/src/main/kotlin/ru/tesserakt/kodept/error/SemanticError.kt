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
        SemanticError("KSeE3", "Usage of undeclared `$name`")

    data class UndefinedUsage(val name: String) :
        SemanticError("KSeE4", "Undefined usage of `$name`") {
        override val additionalMessage = "matches"
    }

    data class DuplicatedVariable(val name: String) :
        SemanticError("KSeE5", "Variable `$name` has duplicates in one block")

    object Duplicated : SemanticError("KSeE6", "Declaration has duplicates across block")

    data class UninitializedUsage(val name: String) :
        SemanticError("KSeE6", "Variable `$name` should be initialized before use")

    data class ImmutableConstruct(val name: String) :
        SemanticError("KSeE7", "Cannot assign to non-assignable structure `$name`")

    data class ImmutableVariable(val name: String) :
        SemanticError("KSeE8", "Cannot assign twice to immutable variable `$name`")

    data class ForeignFunctionParametersTypeMismatch(val name: String) :
        SemanticError("KSeE9", "Function `$name` is foreign, so all its parameters should be foreign too")

    data class ForeignFunctionReturnType(val name: String) :
        SemanticError("KSeE10", "Function `$name` is foreign, so its return type should be foreign or absent")

    data class ForeignFunctionLinkage(val name: String) :
        SemanticError("KSeE11", "There is no known implementation for function `$name`")
}
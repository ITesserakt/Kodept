package ru.tesserakt.kodept.error

sealed class SemanticWarning(override val code: String, override val message: String) : ReportMessage {
    data class NonGlobalSingleModule(val moduleName: String) : SemanticWarning(
        "KW1",
        "Consider replace brackets in module statement `$moduleName` to `=>` operator"
    ) {
        override val additionalMessage: String = "replace with `=>`"
    }
}
package ru.tesserakt.kodept.error

sealed class SemanticWarning(final override val code: String, override val message: String) : ReportMessage {
    init {
        require(code.startsWith("KSeW"))
    }

    data class NonGlobalSingleModule(val moduleName: String) : SemanticWarning(
        "KSeW1",
        "Consider replace brackets in module statement `$moduleName` to `=>` operator"
    ) {
        override val additionalMessage: String = "replace with `=>`"
    }
}
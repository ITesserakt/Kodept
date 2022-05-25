package ru.tesserakt.kodept.error

sealed class SemanticWarning(final override val code: String, override val message: String) : ReportMessage {
    init {
        require(code.startsWith("KSeW"))
    }

    data class NonGlobalSingleModule(val moduleName: String) : SemanticWarning(
        "KSeW1",
        "Consider replace brackets in module statement `$moduleName` to `=>` operator"
    )

    data class EmptyParameterList(val nodeName: String) : SemanticWarning(
        "KSeW2",
        "Remove empty parentheses or add some parameters"
    )

    data class ZeroEnumEntries(val enumName: String) : SemanticWarning(
        "KSeW3",
        "No enum entries declared, remove brackets"
    )

    data class NonUniqueUnionItems(val duplicates: String) : SemanticWarning(
        "KSeW4",
        "Remove duplicated entries in <$duplicates>, they are useless"
    )

    data class AlignWithType(val type: String) : SemanticWarning("KSeW5", "Parentheses around <$type> is useless")
}
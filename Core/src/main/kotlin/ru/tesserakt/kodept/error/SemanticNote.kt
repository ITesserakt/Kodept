package ru.tesserakt.kodept.error

sealed class SemanticNote(final override val code: String, override val message: String) : ReportMessage {
    init {
        require(code.startsWith("KSeN"))
    }

    object EmptyComputationBLock : SemanticNote(
        "KSeN1",
        "Type of empty block is implicitly inferred to ()"
    )

    data class TypeOfFunction(val type: String) : SemanticNote(
        "KSeN2",
        "Type of function inferred to $type"
    )
}
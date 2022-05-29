package ru.tesserakt.kodept.error

data class CompilerCrash(override val message: String) : Exception(message), ReportMessage {
    override val code: String = "KC666"
}
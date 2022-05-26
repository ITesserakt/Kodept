package ru.tesserakt.kodept.error

data class CompilerCrash(override val message: String) : ReportMessage {
    override val code: String = "KC666"
}
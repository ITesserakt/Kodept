package ru.tesserakt.kodept

import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.error.ReportMessage

interface Interpreter<State, Program, Input> {
    fun initialState(input: Input): State
    fun join(state: State, program: Program): State

    fun run(program: Program, input: Input) = join(initialState(input), program)
}

sealed class InterpretationError(final override val code: String) : ReportMessage {
    init {
        require(code.startsWith("KIE"))
    }

    data class RuntimeException(val cause: Throwable) : InterpretationError("KIE1") {
        override val message: String =
            cause.message ?: cause.localizedMessage ?: "Unknown error happened while interpreting"
    }

    data class MultipleMain(val files: List<Filepath>) : InterpretationError("KIE2") {
        override val message: String = "Multiple main functions found in files:\n${
            files.joinToString("\n") { it.name.prependIndent("    ") }
        }"
    }

    object NoMainFunction : InterpretationError("KIE3") {
        override val message: String = "No main function found across all files"
    }

    data class WrongExitCode(val retCode: Int) : InterpretationError("KIE4") {
        override val message: String = "Program finished with exit code: $retCode"
    }
}
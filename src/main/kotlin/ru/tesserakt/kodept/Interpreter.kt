package ru.tesserakt.kodept

interface Interpreter<State, Program, Input> {
    fun initialState(input: Input): State
    fun join(state: State, program: Program): State

    fun run(program: Program, input: Input) = join(initialState(input), program)
}
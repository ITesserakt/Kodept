package ru.tesserakt.kodept.traversal.inference.truely

sealed class Type {
    data class Var(val id: Int) : Type()
    data class Fn(val input: Type, val output: Type) : Type()
    data class Binding(val type: Type) : Type()

    infix fun substitute(other: Type): Type {
        tailrec fun Type.step(other: Type, cnt: Int): Type = when (this) {
            is Var -> when {
                id == 0 && cnt == 0 -> other
                id == 0 && cnt != 0 -> Var(0)
                cnt == 0 -> this
                else -> Var(id - 1).step(other, cnt - 1)
            }
            is Fn -> Fn(input.step(other, cnt), output.step(other, cnt))
            is Binding -> type.step(other, cnt + 1)
        }

        return step(other, 0)
    }

    infix fun isValidIn(context: TypeContext): Boolean = when (this) {
        is Binding -> type isValidIn (context.next())
        is Fn -> input isValidIn context && output isValidIn context
        is Var -> id in context
    }
}
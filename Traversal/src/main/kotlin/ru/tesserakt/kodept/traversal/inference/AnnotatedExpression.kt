package ru.tesserakt.kodept.traversal.inference

import ru.tesserakt.kodept.core.AST

data class AnnotatedExpression(val expr: AST.Expression, val type: Type) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as AnnotatedExpression

        if (expr != other.expr) return false

        return true
    }

    override fun hashCode(): Int {
        return expr.hashCode()
    }
}
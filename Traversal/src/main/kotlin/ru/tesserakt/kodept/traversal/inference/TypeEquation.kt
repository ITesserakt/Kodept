package ru.tesserakt.kodept.traversal.inference

import arrow.core.nel
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.CodePoint
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError

data class TypeEquation(val where: CodePoint, val a: Type, val b: Type) {
    constructor(aexpr: AnnotatedExpression, shouldBe: Type) : this(aexpr.expr.rlt.position, aexpr.type, shouldBe)
    constructor(expr: AST.Expression, type: Type, shouldBe: Type) : this(expr.rlt.position, type, shouldBe)

    private fun occursCheck(v: Type.T, type: Type): Boolean = if (v == type) true
    else if (type is Type.Fn) occursCheck(v, type.input) || occursCheck(v, type.output)
    else false

    context (ReportCollector, Filepath)
    fun unify(): Set<Substitution> = when {
        a is Type.T && occursCheck(a, b) || b is Type.T && occursCheck(b, a) -> {
            report(where.nel(), Report.Severity.ERROR, SemanticError.InfiniteType(a.toString()))
            emptySet()
        }

        a is Type.T -> setOf(Substitution(a, b))
        b is Type.T -> setOf(Substitution(b, a))
        a is Type.Fn && b is Type.Fn -> {
            val s1 = TypeEquation(where, a.input, b.input).unify()
            val s2 = TypeEquation(where, a.output.applySubstitutions(s1), b.output.applySubstitutions(s1)).unify()
            s1 compose s2
        }
        a == b -> emptySet()
        a is Type.Union && b is Type.Union && a.items.containsAll(b.items) && b.items.containsAll(a.items) -> emptySet()
        a is Type.Struct && b is Type.Enum && a.inheritFrom != null && a.inheritFrom.value == b ->
            setOf(Substitution(a, b))
        b is Type.Struct && a is Type.Enum && b.inheritFrom != null && b.inheritFrom.value == a ->
            setOf(Substitution(b, a))
        a is Type.Struct && b is Type.Struct && a.inheritFrom != null && b.inheritFrom != null
                && a.inheritFrom.value == b.inheritFrom.value -> setOf(
            Substitution(a, a.inheritFrom.value),
            Substitution(b, b.inheritFrom.value)
        )
        else -> {
            report(where.nel(), Report.Severity.ERROR, SemanticError.MismatchedType(a.toString(), b.toString()))
            emptySet()
        }
    }
}
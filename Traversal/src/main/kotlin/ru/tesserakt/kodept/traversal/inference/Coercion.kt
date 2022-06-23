@file:Suppress("CANDIDATE_CHOSEN_USING_OVERLOAD_RESOLUTION_BY_LAMBDA_ANNOTATION")

package ru.tesserakt.kodept.traversal.inference

import arrow.core.Valid
import arrow.core.ValidatedNel
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.experimental.ExperimentalTypeInference

interface Coercion {
    context (AST.Expression, ReportCollector, Filepath)
    fun align(a: Type, b: Type): ValidatedNel<Report, Set<Substitution>?>
}

@OptIn(ExperimentalTypeInference::class)
@OverloadResolutionByLambdaReturnType
inline fun Coercion(crossinline f: context(AST.Expression) (Type, Type) -> ValidatedNel<Report, Set<Substitution>?>) =
    object : Coercion {
        context(AST.Expression, ReportCollector, Filepath) override fun align(
            a: Type,
            b: Type,
        ) = f(this@Expression, a, b)
    }

@JvmName("CoercionJust")
inline fun Coercion(crossinline f: context(AST.Expression) (Type, Type) -> Set<Substitution>?) =
    object : Coercion {
        context(AST.Expression, ReportCollector, Filepath) override fun align(
            a: Type,
            b: Type,
        ) = Valid(f(this@Expression, a, b))
    }

object DefaultCoercions {
    val generic = Coercion { a, b ->
        if (a is Type.T) setOf(Substitution(a, b))
        else if (b is Type.T) setOf(Substitution(b, a))
        else null
    }

    val equality = Coercion { a: Type, b: Type ->
        if (a == b) emptySet()
        else null
    }

    val unionCombination = Coercion { a: Type, b: Type ->
        if (a !is Type.Union || b !is Type.Union) null
        else if (a.items.containsAll(b.items) && b.items.containsAll(a.items)) emptySet()
        else null
    }

    val all = arrayListOf(generic, equality, unionCombination)
}
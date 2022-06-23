package ru.tesserakt.kodept.traversal.inference

import arrow.core.identity
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.walkDownTop

fun groupByFunctions(types: List<AnnotatedExpression>) =
    types.groupBy { it.expr.walkDownTop(::identity).filterIsInstance<AST.FunctionLike>().first() }


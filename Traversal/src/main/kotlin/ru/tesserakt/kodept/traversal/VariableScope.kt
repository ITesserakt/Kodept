package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.identity
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.core.walkDownTop
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

object VariableScope : SpecificTransformer<AST.InitializedVar>() {
    override val type: KClass<AST.InitializedVar> = AST.InitializedVar::class

    init {
        dependsOn(objectUniqueness)
    }

    context(ReportCollector, Filename) override fun transformTo(node: AST.InitializedVar): EagerEffect<UnrecoverableError, Pair<AST.Node, AST.Node>> {
        val nearestBlock = node.walkDownTop(::identity).filterIsInstance<AST.ExpressionList>().first()
        val varIndex = nearestBlock.expressions.indexOf(node)
        val (outer, inner) = nearestBlock.expressions.withIndex().partition { it.index < varIndex }
        val scope = AST.ExpressionList(inner.map { it.value })
        return eagerEffect { nearestBlock to AST.ExpressionList(outer.map { it.value } + scope) }
    }
}
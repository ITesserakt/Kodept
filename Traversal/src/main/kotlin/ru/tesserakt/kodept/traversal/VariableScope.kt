package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.identity
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.core.walkDownTop
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

object VariableScope : Transformer<AST.InitializedVar>() {
    override val type: KClass<AST.InitializedVar> = AST.InitializedVar::class

    init {
        dependsOn(objectUniqueness)
    }

    context(ReportCollector, Filename) override fun transform(node: AST.InitializedVar): EagerEffect<UnrecoverableError, out AST.Node> {
        val nearestBlock = node.walkDownTop(::identity).filterIsInstance<AST.ExpressionList>().first()
        val varIndex = nearestBlock.expressions.indexOf(node)
        val (outer, inner) = nearestBlock.expressions.withIndex().partition { it.index < varIndex }
        val scope = AST.ExpressionList(inner.map { it.value })
        nearestBlock.parent!!.replaceChild(nearestBlock, AST.ExpressionList(outer.map { it.value } + scope))
        return eagerEffect { node }
    }
}
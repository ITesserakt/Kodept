package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import arrow.core.identity
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.walkDownTop
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

object VariableScope : SpecificTransformer<AST.InitializedVar>() {
    override val type: KClass<AST.InitializedVar> = AST.InitializedVar::class

    init {
        dependsOn(objectUniqueness)
    }

    context(ReportCollector, Filepath) override fun transformTo(node: AST.InitializedVar): EagerEffect<UnrecoverableError, Pair<AST.Node, AST.Node>> {
        val nearestBlock = node.walkDownTop(::identity).filterIsInstance<AST.ExpressionList>().first()
        val varIndex = nearestBlock.expressions.indexOf(node)
        if (varIndex == -1) println("Warn: var not found")
        val (outer, inner) = nearestBlock.expressions.withIndex().partition { it.index < varIndex }

        if (outer.isEmpty()) return eagerEffect { nearestBlock to nearestBlock }

        val scope = nearestBlock.copy(inner.map { it.value })
        return eagerEffect { nearestBlock to nearestBlock.copy(outer.map { it.value } + scope) }
    }
}
package ru.tesserakt.kodept.traversal

import arrow.core.NonEmptyList
import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.core.Tree
import ru.tesserakt.kodept.core.move
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

object VariableScope : Transformer<AST.ExpressionList>() {
    override val type: KClass<AST.ExpressionList> = AST.ExpressionList::class

    init {
        dependsOn(objectUniqueness)
    }

    override val traverseMode: Tree.SearchMode = Tree.SearchMode.LevelOrder

    private fun AST.ExpressionList.splitBlock(varIndex: Int): Pair<NonEmptyList<AST.BlockLevel>, AST.ExpressionList>? {
        val (outerI, innerI) = expressions.withIndex().partition { it.index < varIndex }
        val (outer, inner) = outerI.map { it.value } to innerI.map { it.value }

        if (outer.isEmpty()) return null
        if (inner.size == 1) return null

        return with(rlt) {
            NonEmptyList.fromListUnsafe(outer) to
                    AST.ExpressionList(NonEmptyList.fromListUnsafe(inner.move())).withRLT()
        }
    }

    private tailrec fun step(currentBlock: AST.ExpressionList, skips: List<Int>): AST.ExpressionList {
        val varIndex = currentBlock.expressions.withIndex()
            .indexOfFirst { it.value is AST.InitializedVar && it.index !in skips }
        if (varIndex == -1) return currentBlock
        val split = currentBlock.splitBlock(varIndex)

        return if (split == null) step(currentBlock, skips + varIndex)
        else with(currentBlock.rlt) {
            AST.ExpressionList(split.first.map { it.move() } + step(split.second, emptyList()).move()).withRLT()
        }
    }

    context(ReportCollector, Filepath) override fun transform(node: AST.ExpressionList): EagerEffect<UnrecoverableError, out AST.Node> {
        return eagerEffect { step(node, emptyList()) }
    }
}
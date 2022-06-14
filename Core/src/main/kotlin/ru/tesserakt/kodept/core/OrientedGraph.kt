package ru.tesserakt.kodept.core

import arrow.core.*
import arrow.core.continuations.EagerEffectScope
import arrow.core.continuations.either
import kotlin.collections.component1
import kotlin.collections.component2
import kotlin.collections.set

class OrientedGraph<T> private constructor() {
    sealed interface Errors
    data class Cycle<T>(val inside: NonEmptyList<T>) : Errors {
        constructor(item: T, vararg items: T) : this(nonEmptyListOf(item, *items))
    }

    object NotFound : Errors

    interface Node<Self : Node<Self>> {
        fun children(): List<Self>
    }

    private enum class Color { NotVisited, Visited, Processed }
    private sealed interface TopSortLink

    @JvmInline
    private value class Existing(val value: Int) : TopSortLink {
        operator fun invoke() = value
        operator fun minus(other: Int) = if (value - other > 0) Existing(value - other) else Free
    }

    private object Free : TopSortLink
    private object NonExisting : TopSortLink

    private var matrix: Array<BooleanArray> = emptyArray()
    private val nodes: MutableMap<Int, T> = mutableMapOf()
    private var id = 0

    private fun growMatrix() {
        if (matrix.size < id) {
            val newMatrix = Array(id.coerceAtLeast(1)) { BooleanArray(id.coerceAtLeast(1)) { false } }
            for ((i, row) in matrix.withIndex())
                for ((j, cell) in row.withIndex())
                    newMatrix[i][j] = cell
            matrix = newMatrix
        }
    }

    private fun findByValueOrAssign(value: T) =
        nodes.entries.firstOrNull { it.value == value }?.let { it.key to it.value } ?: (id++ to value)

    fun T.addChildren(vararg child: T) {
        val (k, v) = findByValueOrAssign(this)
        val rest = child.map(::findByValueOrAssign)
        (k to v).prependTo(rest).forEach { nodes[it.first] = it.second }

        growMatrix()
        for ((j, _) in rest)
            matrix[k][j] = true
    }

    fun T.addNode() {
        val (k, v) = findByValueOrAssign(this)
        nodes[k] = v
        growMatrix()
    }

    private fun dfs(start: Int, f: (T) -> Unit): Either<Errors, Unit> {
        val visitMap: MutableMap<Int, Color> = mutableMapOf()

        suspend fun EagerEffectScope<Errors>.step(from: Int) {
            visitMap[from] = Color.Visited
            matrix[from].withIndex().filter { it.value }.forEach { (j, _) ->
                when (visitMap.getOrDefault(j, Color.NotVisited)) {
                    Color.NotVisited -> step(j)
                    Color.Visited -> shift<Nothing>(Cycle(nonEmptyListOf(nodes[j])))
                    Color.Processed -> Unit
                }
            }
            f(nodes[from] ?: throw IllegalStateException())
            visitMap[from] = Color.Processed
        }

        return either.eager { step(start) }
    }

    private fun Array<BooleanArray>.transpose(): Array<BooleanArray> {
        val cols = this.firstOrNull()?.size ?: return emptyArray()
        val rows = this.size
        return Array(cols) { j ->
            BooleanArray(rows) { i -> this[i][j] }
        }
    }

    fun sortedLayers() = either.eager<Errors, List<List<T>>> {
        val sums = matrix.transpose().mapIndexed { index, it ->
            val cnt = it.count(::identity)
            if (cnt == 0) IndexedValue(index, Free)
            else IndexedValue(index, Existing(cnt))
        }.associate { it.index to it.value }.toMutableMap()

        buildList {
            while (!sums.all { it.value == NonExisting }) {
                val layer = sums.filterValues { it == Free }.keys
                if (layer.isEmpty()) shift<Nothing>(Cycle(NonEmptyList.fromListUnsafe(
                    sums.filterValues { it is Existing && it.value == 1 }.map { nodes[it.key] })
                )
                )
                add(layer.map { nodes[it] ?: shift<Nothing>(NotFound) })

                val impacts = layer.flatMap { node -> matrix[node].withIndex().filter { it.value }.map { it.index } }
                for (i in impacts)
                    when (val link = sums[i]) {
                        is Existing -> sums[i] = link - 1
                        else -> shift<Nothing>(NotFound)
                    }
                layer.forEach { sums[it] = NonExisting }
            }
        }
    }

    fun topSort(start: T) = either.eager<Errors, List<T>> {
        val (i, _) = nodes.entries.find { it.value == start } ?: shift<Nothing>(NotFound)
        val stack = ArrayDeque<T>(nodes.size)
        dfs(i) { stack.addLast(it) }.bind()
        stack.asReversed()
    }

    companion object {
        operator fun <T> invoke(scope: OrientedGraph<T>.() -> Unit) = OrientedGraph<T>().also(scope)

        inline fun <reified T : Node<T>> fromNodes(nodes: Iterable<T>) = invoke {
            nodes.forEach { it.addChildren(*it.children().toTypedArray()) }
        }

        inline fun <reified T : Tree<T>> fromTree(root: T) = invoke {
            root.walkTopDown {
                it.addChildren(*it.children().toTypedArray())
            }.forEach { _ -> }
        }
    }
}
package ru.tesserakt.kodept.traversal

import ru.tesserakt.kodept.core.OrientedGraph

sealed class Depended : OrientedGraph.Node<Depended> {
    private val depended = mutableListOf<Depended>()

    protected fun dependsOn(vararg parent: Depended) {
        parent.forEach { it.depended += this }
    }

    override fun children(): List<Depended> = depended.toList()
}

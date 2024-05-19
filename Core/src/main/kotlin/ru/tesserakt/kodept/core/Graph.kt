package ru.tesserakt.kodept.core

@JvmInline
value class Graph<T> private constructor(private val inner: OrientedGraph<T>) {
    fun T.addChildren(vararg child: T) {
        with(inner) {
            child.forEach {
                this@addChildren.addChildren(it)
                it.addChildren(this@addChildren)
            }
        }
    }

    fun T.addNode() = with(inner) { addNode() }

    companion object {
        operator fun <T> invoke(block: Graph<T>.() -> Unit) = Graph<T>(OrientedGraph { }).apply(block)
    }
}
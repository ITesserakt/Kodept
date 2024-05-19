package ru.tesserakt.kodept.core

import arrow.core.identity

@Suppress("unused", "UNCHECKED_CAST")
interface Tree<Self : Tree<Self>> : OrientedGraph.Node<Self> {
    val parent: Self?

    enum class SearchMode {
        Postorder {
            override suspend fun <T : Tree<T>> SequenceScope<T>.acquire(initial: T) {
                val stack = ArrayDeque(arrayListOf(initial))
                var root = initial
                while (stack.isNotEmpty()) {
                    val current = stack.last()
                    val children = current.children()
                    if (children.isEmpty() || children.any { it == root }) {
                        yield(stack.removeLast())
                        root = current
                    } else {
                        stack += children.asReversed()
                    }
                }
            }
        },
        Preorder {
            override suspend fun <T : Tree<T>> SequenceScope<T>.acquire(initial: T) {
                val stack = ArrayDeque(arrayListOf(initial))
                while (stack.isNotEmpty()) {
                    val current = stack.removeLast()
                    stack += current.children().asReversed()
                    yield(current)
                }
            }
        },
        LevelOrder {
            override suspend fun <T : Tree<T>> SequenceScope<T>.acquire(initial: T) {
                val queue = ArrayDeque(arrayListOf(initial))
                while (queue.isNotEmpty()) {
                    val current = queue.removeFirst()
                    queue += current.children()
                    yield(current)
                }
            }
        };

        abstract suspend fun <T : Tree<T>> SequenceScope<T>.acquire(initial: T)
    }

    fun <T> walkTopDown(mode: SearchMode = SearchMode.LevelOrder, f: (Self) -> T) = with(mode) {
        sequence { acquire(this@Tree as Self) }.map(f)
    }

    fun gatherChildren(mode: SearchMode = SearchMode.LevelOrder) = walkTopDown(mode, ::identity)
}

inline fun <Self : Tree<Self>, T> Self.walkDownTop(crossinline f: (Self) -> T) = sequence {
    var current: Self? = this@walkDownTop
    while (current != null) {
        yield(f(current))
        current = current.parent
    }
}
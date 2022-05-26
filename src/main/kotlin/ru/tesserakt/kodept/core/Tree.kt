package ru.tesserakt.kodept.core

import java.util.*
import kotlin.collections.ArrayDeque
import kotlin.collections.List
import kotlin.collections.asReversed
import kotlin.collections.forEach
import kotlin.collections.isNotEmpty
import kotlin.collections.plusAssign

@Suppress("unused", "UNCHECKED_CAST")
interface Tree<Self : Tree<Self>> {
    val parent: Self?
    fun children(): List<Self>

    enum class SearchMode {
        Postorder {
            override suspend fun <T : Tree<T>> SequenceScope<T>.acquire(initial: T) {
                fun step(current: T): Sequence<T> = sequence {
                    current.children().forEach { yieldAll(step(it)) }
                    yield(current)
                }
                yieldAll(step(initial))
            }
        },
        Preorder {
            override suspend fun <T : Tree<T>> SequenceScope<T>.acquire(initial: T) {
                val stack = Stack<T>()
                stack += initial
                while (stack.isNotEmpty()) {
                    val current = stack.pop()
                    stack += current.children().asReversed()
                    yield(current)
                }
            }
        },
        LevelOrder {
            override suspend fun <T : Tree<T>> SequenceScope<T>.acquire(initial: T) {
                val queue = ArrayDeque<T>(1)
                queue += initial
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
}

inline fun <Self : Tree<Self>, T> Self.walkDownTop(crossinline f: (Self) -> T) = sequence {
    var current: Self? = this@walkDownTop
    while (current != null) {
        yield(f(current))
        current = current.parent
    }
}
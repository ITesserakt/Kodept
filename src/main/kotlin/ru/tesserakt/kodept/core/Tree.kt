package ru.tesserakt.kodept.core

@Suppress("unused", "UNCHECKED_CAST")
abstract class Tree<Self : Tree<Self>> {
    abstract val parent: Self?
    abstract fun children(): List<Self>

    enum class SearchMode {
        Inorder {
            override fun <T : Tree<T>> MutableList<T>.acquire() = removeLast().also {
                this += it.children().asReversed()
            }
        },
        Postorder {
            override fun <T : Tree<T>> MutableList<T>.acquire() = removeLast().also {
                this += it.children()
            }
        },
        Preorder {
            override fun <T : Tree<T>> MutableList<T>.acquire() = removeFirst().also {
                this += it.children().asReversed()
            }
        },
        LevelOrder {
            override fun <T : Tree<T>> MutableList<T>.acquire() = removeFirst().also {
                this += it.children()
            }
        };

        abstract fun <T : Tree<T>> MutableList<T>.acquire(): T
    }

    @Suppress("TYPE_MISMATCH_WARNING")
    inline fun <T> walkTopDown(mode: SearchMode = SearchMode.LevelOrder, crossinline f: (Self) -> T) =
        sequence {
            val queue = arrayListOf(this@Tree)
            while (queue.isNotEmpty()) {
                val current = with(mode) { queue.acquire() }
                yield(f(current as Self))
            }
        }

    inline fun <T> walkDownTop(crossinline f: (Self) -> T) = sequence {
        var current: Tree<Self>? = this@Tree
        while (current != null) {
            yield(f(current as Self))
            current = current.parent
        }
    }
}
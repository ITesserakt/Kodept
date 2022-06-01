package ru.tesserakt.kodept.core

import io.kotest.assertions.arrow.core.shouldBeLeft
import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.StringSpec

class AlgorithmsTest : StringSpec({
    val graph = OrientedGraph {
        'a'.addChildren('b', 'c', 'd', 'e')
        'b'.addChildren('d')
        'c'.addChildren('d', 'e')
        'd'.addChildren('e')
    }

    "topSort on full graph" {
        graph.sortedLayers() shouldBeRight listOf("a", "bc", "d", "e").map { it.toList() }

        graph.apply { 'u'.addNode() }
        graph.sortedLayers() shouldBeRight listOf("au", "bc", "d", "e").map { it.toList() }

        graph.apply { 'u'.addChildren('v') }
        graph.sortedLayers() shouldBeRight listOf("au", "bcv", "d", "e").map { it.toList() }

        graph.apply {
            'v'.addChildren('a')
            'a'.addChildren('u')
        }
        graph.sortedLayers() shouldBeLeft OrientedGraph.Cycle
    }

    "topSort with root node" {
        graph.topSort('v') shouldBeLeft OrientedGraph.Cycle

        graph.topSort('c') shouldBeRight "cde".toList()

        graph.topSort('n') shouldBeLeft OrientedGraph.NotFound
    }
})
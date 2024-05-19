package ru.tesserakt.kodept.core

import arrow.core.raise.toEither
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
        graph.sortedLayers().toEither() shouldBeRight listOf("a", "bc", "d", "e").map { it.toList() }

        graph.apply { 'u'.addNode() }
        graph.sortedLayers().toEither() shouldBeRight listOf("au", "bc", "d", "e").map { it.toList() }

        graph.apply { 'u'.addChildren('v') }
        graph.sortedLayers().toEither() shouldBeRight listOf("au", "bcv", "d", "e").map { it.toList() }

        graph.apply {
            'v'.addChildren('a')
            'a'.addChildren('u')
        }
        graph.sortedLayers().toEither() shouldBeLeft OrientedGraph.Cycle('a', 'b', 'c', 'u', 'v')
    }

    "topSort with root node" {
        graph.topSort('v').toEither() shouldBeLeft OrientedGraph.Cycle('v')

        graph.topSort('c').toEither() shouldBeRight "cde".toList()

        graph.topSort('n').toEither() shouldBeLeft OrientedGraph.NotFound
    }
})
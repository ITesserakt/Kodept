package ru.tesserakt.kodept.core

interface Emitter {
    fun emit(node: AST.Node)
}

class KodeptEmitter : Emitter {
    override fun emit(node: AST.Node) {

    }
}
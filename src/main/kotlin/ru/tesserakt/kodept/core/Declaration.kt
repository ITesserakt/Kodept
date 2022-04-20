package ru.tesserakt.kodept.core

data class Declaration(val decl: AST.Node, val parent: Declaration?, val name: String)


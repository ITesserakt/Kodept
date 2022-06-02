package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

object TypeDereferenceTransformer : Transformer<AST.TypeReference>() {
    override val type: KClass<AST.TypeReference> = AST.TypeReference::class

    /**
     * This works except for [AST.Dereference]: we should know type of the left branch to find proper reference for the right branch
     *
     * Every type reference is in block: [AST.ExpressionList] or [AST.FunctionDecl]
     *
     * 1. reference without context:
     *
     */
    context(ReportCollector, Filename) override fun transform(node: AST.TypeReference): EagerEffect<UnrecoverableError, out AST.Node> =
        eagerEffect {

            node
        }
}
package ru.tesserakt.kodept.traversal

import arrow.core.continuations.EagerEffect
import arrow.core.continuations.eagerEffect
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.InsecureModifications.setRawLexem
import ru.tesserakt.kodept.core.move
import ru.tesserakt.kodept.core.new
import ru.tesserakt.kodept.error.ReportCollector
import kotlin.reflect.KClass

object StructAllocGetters : Transformer<AST.StructDecl>() {
    override val type: KClass<AST.StructDecl> = AST.StructDecl::class

    private fun generateGetter(struct: AST.StructDecl, variable: AST.Parameter) = AST.FunctionDecl(
        variable.name,
        listOf(
            AST.Parameter("self", AST.ResolvedTypeReference(struct.name, struct).setRawLexem(struct.rlt))
                .setRawLexem(struct.rlt)
        ).move(),
        variable.type.new(),
        AST.Intrinsics.AccessVariable(struct, variable).setRawLexem(variable.rlt).move()
    ).setRawLexem(variable.rlt)

    context(ReportCollector, Filepath) override fun transform(node: AST.StructDecl): EagerEffect<UnrecoverableError, out AST.Node> =
        eagerEffect {
            val functionLikeGetters = node.rest.filterIsInstance<AST.FunctionLike>().map { it.name }
            val toGenerate = node.alloc.filter { it.name !in functionLikeGetters }
            val getters = toGenerate.map { generateGetter(node, it) }
            if (getters.isEmpty()) node
            else AST.StructDecl(node.name, node.alloc, node.rest + getters).setRawLexem(node.rlt)
        }
}
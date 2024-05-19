package ru.tesserakt.kodept.traversal

import arrow.core.raise.EagerEffect
import arrow.core.raise.eagerEffect
import arrow.core.ensureThat
import arrow.core.nel
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.InsecureModifications.setRawLexem
import ru.tesserakt.kodept.core.move
import ru.tesserakt.kodept.core.new
import ru.tesserakt.kodept.error.Report
import ru.tesserakt.kodept.error.ReportCollector
import ru.tesserakt.kodept.error.SemanticError
import kotlin.reflect.KClass

object StructCtorGenerator : Transformer<AST.StructDecl>() {
    override val type: KClass<AST.StructDecl> = AST.StructDecl::class

    private fun generateCtor(struct: AST.StructDecl, alloc: List<AST.Parameter>): AST.FunctionDecl {
        val params = alloc.new()

        return AST.FunctionDecl(
            "new",
            params,
            AST.ResolvedTypeReference(struct.name, struct).setRawLexem(struct.rlt).move(),
            AST.Intrinsics.Construct(struct, params.map { AST.ResolvedReference(it.value.name, it).setRawLexem(it.rlt) })
                .setRawLexem(struct.rlt).move()
        ).setRawLexem(struct.rlt)
    }

    context(ReportCollector, Filepath) override fun transform(node: AST.StructDecl): EagerEffect<UnrecoverableError, out AST.Node> =
        eagerEffect {
            val anyCtor = node.rest.filterIsInstance<AST.FunctionLike>().find { it.name == "new" }

            ensureThat(anyCtor == null || anyCtor is AST.FunctionDecl && "self" !in anyCtor.params.map(AST.InferredParameter::name)) {
                UnrecoverableError(
                    anyCtor?.rlt?.position?.nel(),
                    Report.Severity.ERROR,
                    SemanticError.WrongConstructor(node.name)
                )
            }

            if (anyCtor != null) node
            else AST.StructDecl(node.name, node.alloc, node.rest + generateCtor(node, node.alloc)).setRawLexem(node.rlt)
        }
}
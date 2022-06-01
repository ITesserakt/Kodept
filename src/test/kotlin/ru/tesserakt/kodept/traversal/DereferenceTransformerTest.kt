package ru.tesserakt.kodept.traversal

import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.types.shouldBeTypeOf
import io.mockk.clearAllMocks
import io.mockk.mockk
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filename
import ru.tesserakt.kodept.parser.RLT

class DereferenceTransformerTest : StringSpec() {
    context(Filename) private fun buildAST(root: AST.Node) = AST(root, this@Filename).apply {
        walkThrough {
            when (it) {
                is AST.Reference -> it._rlt = mockk<RLT.Reference>(relaxed = true)
                is AST.InitializedVar -> it._rlt = mockk<RLT.InitializedAssignment>(relaxed = true)
                is AST.VariableDecl -> it._rlt = mockk<RLT.Variable>(relaxed = true)
            }
        }.forEach { _ -> }
    }

    init {
        with("TEST FILE" as Filename) {
            "reference by variable declaration" {
                val ref = AST.Reference("x")
                val variable = AST.VariableDecl(ref, false, null)
                buildAST(variable)

                unwrap { DereferenceTransformer.transform(ref) }.toEither()
                    .shouldBeRight(AST.ResolvedReference(ref, variable))
            }

            "reference by initialized variable declaration" {
                val ref = AST.Reference("x")
                val initVar = AST.InitializedVar(ref, false, null, AST.DecimalLiteral(5.toBigInteger()))
                buildAST(initVar)

                unwrap { DereferenceTransformer.transform(ref) }.toEither()
                    .shouldBeRight(AST.ResolvedReference(ref, initVar))
            }

            "reference by variable declaration somewhere in a block" {
                val ref = AST.Reference("x")
                val block = AST.ExpressionList(
                    listOf(
                        AST.VariableDecl(AST.Reference("y"), true, null),
                        AST.VariableDecl(AST.Reference("x"), false, null),
                        ref
                    )
                )
                buildAST(block)

                unwrap { DereferenceTransformer.transform(ref) }.toEither()
                    .shouldBeRight(AST.ResolvedReference(ref, AST.VariableDecl(ref, false, null)))
            }

            "reference by function name somewhere in a block" {
                val ref = AST.Reference("x")
                val block = AST.ExpressionList(
                    listOf(
                        AST.ExpressionList(listOf(ref)),
                        AST.FunctionDecl("x", emptyList(), null, AST.TupleLiteral.unit)
                    )
                )
                buildAST(block)

                unwrap { DereferenceTransformer.transform(ref) }
                    .toEither().shouldBeRight()
                    .shouldBeTypeOf<AST.ResolvedReference>()
                    .referral.shouldBeTypeOf<AST.FunctionDecl>()
            }
        }

        afterSpec { clearAllMocks() }
    }
}
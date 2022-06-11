package ru.tesserakt.kodept.traversal

import io.kotest.assertions.arrow.core.shouldBeRight
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.types.shouldBeTypeOf
import io.mockk.clearAllMocks
import io.mockk.mockk
import ru.tesserakt.kodept.core.AST
import ru.tesserakt.kodept.core.Filepath
import ru.tesserakt.kodept.core.InsecureModifications.withRLT
import ru.tesserakt.kodept.core.RLT

class DereferenceTransformerTest : StringSpec() {
    private fun Filepath.buildAST(root: AST.Node) = AST(root, this).apply {
        walkThrough {
            when (it) {
                is AST.Reference -> with(mockk<RLT.Reference>(relaxed = true)) { it.withRLT() }
                is AST.InitializedVar -> with(mockk<RLT.InitializedAssignment>(relaxed = true)) { it.withRLT() }
                else -> Unit
            }
        }.forEach { _ -> }
    }

    init {
        with(Filepath("TEST FILE")) {
            "reference by initialized variable declaration" {
                val ref = AST.Reference("x")
                val initVar = AST.InitializedVar(ref, false, null, AST.DecimalLiteral(5.toBigInteger()))
                buildAST(initVar)

                unwrap { DereferenceTransformer.transform(ref) }.toEither()
                    .shouldBeRight(AST.ResolvedReference(ref.name, initVar, ref.resolutionContext))
            }

            "reference by variable declaration somewhere in a block" {
                val ref = AST.Reference("x")
                val block = AST.ExpressionList(
                    listOf(
                        AST.InitializedVar(AST.Reference("y"), true, null, AST.StringLiteral("test")),
                        AST.InitializedVar(AST.Reference("x"), false, null, AST.CharLiteral('y')),
                        ref
                    )
                )
                buildAST(block)

                unwrap { DereferenceTransformer.transform(ref) }.toEither()
                    .shouldBeRight(
                        AST.ResolvedReference(
                            ref.name,
                            AST.InitializedVar(ref, false, null, AST.CharLiteral('y')),
                            ref.resolutionContext
                        )
                    )
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